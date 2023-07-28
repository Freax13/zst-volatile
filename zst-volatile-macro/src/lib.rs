use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, punctuated::Punctuated, Attribute, Data, DeriveInput, Error, Lit, Meta,
    NestedMeta, Token,
};

#[proc_macro_derive(VolatileStruct)]
pub fn derive_volatile(tts: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tts as DeriveInput);

    let Data::Struct(strukt) = &input.data else {
        return Error::new_spanned(&input,"only structs are supported").into_compile_error().into();
    };

    let (repr_c, repr_packed) = parse_repr(&input.attrs).unwrap();
    if !repr_c {
        return Error::new_spanned(&input, "VolatileStruct needs #[repr(C)]")
            .into_compile_error()
            .into();
    }

    let s_vis = &input.vis;
    let s_ident = &input.ident;
    let volatile_ident = format_ident!("Volatile{}", s_ident);

    let mut offset = quote! { ::zst_volatile::offset::Zero };
    let mut field_wrappers = Vec::new();
    let mut fields = Vec::new();

    for f in &strukt.fields {
        let f_vis = &f.vis;
        // FIXME: Make this compatible with unnamed structs.
        let f_ident = &f.ident.as_ref().unwrap();
        let f_ty = &f.ty;

        if let Some(n) = repr_packed {
            // Generate a unique non-aligned wrapper type for each field
            let f_wrapper = format_ident!("{}_{}", s_ident, f_ident);
            let lit_n = Literal::usize_unsuffixed(n);

            field_wrappers.push(quote! {
                #[allow(non_camel_case_types)]
                #[repr(C, packed(#lit_n))]
                #f_vis struct #f_wrapper(#f_ty);
            });

            offset = quote! {
                ::zst_volatile::offset::Align::<#offset, #f_wrapper>
            };
        } else {
            offset = quote! {
                ::zst_volatile::offset::Align::<#offset, #f_ty>
            };
        }

        fields.push(quote! {
            #f_vis #f_ident: ::zst_volatile::Volatile<#f_ty, #offset>
        });

        // Calculate the offset for the next field.
        offset = quote! {
            ::zst_volatile::offset::PastField::<#offset, #f_ty>
        };
    }

    quote! {
        #(#field_wrappers)*

        #s_vis struct #volatile_ident {
            #(#fields,)*
        }

        unsafe impl ::zst_volatile::VolatileStruct for #s_ident {
            type Struct = #volatile_ident;
        }
    }
    .into()
}

fn parse_repr(attrs: &[Attribute]) -> syn::Result<(bool, Option<usize>)> {
    let mut repr_c = false;
    let mut repr_packed = None::<usize>;
    for a in attrs {
        if a.path.is_ident("repr") {
            for meta in a.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)? {
                match meta {
                    Meta::Path(path) if path.is_ident("C") => repr_c = true,
                    Meta::Path(path) if path.is_ident("packed") => repr_packed = Some(1),
                    Meta::List(metas) if meta.path().is_ident("packed") => {
                        let nested: Vec<_> = metas.nested.iter().collect();
                        match nested.as_slice() {
                            &[NestedMeta::Lit(Lit::Int(i))] => {
                                let n: usize = i.base10_parse()?;
                                if let Some(old) = repr_packed {
                                    if n < old {
                                        repr_packed = Some(n)
                                    }
                                }
                            }
                            _ => {} // Wrong packed format
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok((repr_c, repr_packed))
}
