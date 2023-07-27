use proc_macro::TokenStream;
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

    let (repr_c, repr_align, repr_packed) = parse_repr(&input.attrs).unwrap();
    if !repr_c {
        panic!("VolatileStruct needs #[repr(C)]")
    }

    let vis = &input.vis;
    let ident = &input.ident;
    let volatile_ident = format_ident!("Volatile{}", ident);

    let mut offset = quote! { ::zst_volatile::offset::Zero };
    let fields = strukt.fields.iter().map(|field| {
        let vis = &field.vis;
        // FIXME: Make this compatible with unnamed structs.
        let ident = &field.ident.as_ref().unwrap();
        let ty = &field.ty;

        // FIXME: How to process #[repr(align(N))] and #[repr(packed(N))]?
        // if let Some(pack_n) = repr_packed {
        //     // Align the field.
        //     offset = quote! {
        //         ::zst_volatile::offset::Align::<#offset, #ty>
        //     };
        // }

        let field = quote! {
            #vis #ident: ::zst_volatile::Volatile<#ty, #offset>
        };

        // Calculate the minimum offset for the next field.
        offset = quote! {
            ::zst_volatile::offset::PastField::<#offset, #ty>
        };

        field
    });

    quote! {
        #vis struct #volatile_ident {
            #(#fields,)*
        }

        unsafe impl ::zst_volatile::VolatileStruct for #ident {
            type Struct = #volatile_ident;
        }
    }
    .into()
}

fn parse_repr(attrs: &[Attribute]) -> syn::Result<(bool, Option<usize>, Option<usize>)> {
    let mut repr_c = false;
    let mut repr_align = None::<usize>;
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
                                repr_packed = Some(n)
                            }
                            _ => {} // Wrong packed format
                        }
                    }
                    Meta::List(metas) if meta.path().is_ident("align") => {
                        let nested: Vec<_> = metas.nested.iter().collect();
                        match nested.as_slice() {
                            &[NestedMeta::Lit(Lit::Int(i))] => {
                                let n: usize = i.base10_parse()?;
                                repr_align = Some(n)
                            }
                            _ => {} // Wrong align format
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok((repr_c, repr_align, repr_packed))
}
