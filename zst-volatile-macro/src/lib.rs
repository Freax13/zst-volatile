use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal, Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Error, Lit, Meta, NestedMeta, Result,
    Visibility,
};

#[proc_macro_derive(VolatileStruct)]
pub fn derive_volatile(tts: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tts as DeriveInput);

    let Data::Struct(strukt) = &input.data else {
        return Error::new_spanned(&input, "only structs are supported")
            .into_compile_error()
            .into();
    };

    let packed = match check_repr(&input.attrs) {
        Ok(packed) => packed,
        Err(err) => return err.into_compile_error().into(),
    };

    let vis = &input.vis;
    let ident = &input.ident;
    let volatile_ident = format_ident!("Volatile{}", ident);

    let (alignment_defs, alignment_ty) = match packed {
        Some(packed) => generate_packed_wrapper(vis, ident, packed),
        None => (quote! {}, quote! { A }),
    };

    let mut offset = quote! { ::zst_volatile::offset::Zero };
    let fields = strukt.fields.iter().map(|field| {
        let vis = &field.vis;
        // FIXME: Make this compatible with unnamed structs.
        let ident = &field.ident.as_ref().unwrap();
        let ty = &field.ty;

        // Align the field.
        let field_offset = quote! {
            ::zst_volatile::offset::Align::<#offset, <#alignment_ty as ::zst_volatile::alignment::Alignment>::Wrapper<#ty>>
        };

        // Calculate the minimum offset for the next field.
        offset = quote! {
            ::zst_volatile::offset::PastField::<#field_offset, #ty>
        };

        quote! {
            #vis #ident: ::zst_volatile::Volatile<#ty, #field_offset, #alignment_ty>
        }
    });

    quote! {
        #vis struct #volatile_ident<A = ::zst_volatile::alignment::Normal>
        where
            A: ::zst_volatile::alignment::Alignment,
        {
            #(#fields,)*
            _alignment_marker: ::core::marker::PhantomData<A>,
        }

        unsafe impl ::zst_volatile::VolatileStruct for #ident {
            type Struct<A> = #volatile_ident<A>
            where
                A: ::zst_volatile::alignment::Alignment;
        }

        #alignment_defs
    }
    .into()
}

fn check_repr(attrs: &[Attribute]) -> Result<Option<usize>> {
    let mut is_c_repr = false;
    let mut packed = None;

    for attr in attrs.iter().filter(|a| a.path.is_ident("repr")) {
        let meta = attr.parse_meta()?;
        let Meta::List(list) = meta else {
            return Err(Error::new_spanned(meta, "expected list"));
        };
        for nested_meta in list.nested {
            let NestedMeta::Meta(m) = nested_meta else {
                return Err(Error::new_spanned(nested_meta, "expected meta"));
            };
            if m.path().is_ident("C") {
                is_c_repr = true;
            } else if m.path().is_ident("packed") {
                match m {
                    Meta::Path(_) => packed = Some(1),
                    Meta::List(list) => {
                        if list.nested.len() != 1 {
                            return Err(Error::new_spanned(list, "expected exactly one value"));
                        }
                        let first = list.nested.first().unwrap();
                        let NestedMeta::Lit(lit) = first else {
                            return Err(Error::new_spanned(list, "expected literal"));
                        };
                        let Lit::Int(lit_int) = lit else {
                            return Err(Error::new_spanned(list, "expected integer literal"));
                        };
                        let value = lit_int.base10_parse()?;
                        packed = Some(value);
                    }
                    Meta::NameValue(_) => {
                        return Err(Error::new_spanned(m, "expected path or list meta"));
                    }
                }
            } else if m.path().is_ident("align") {
                // Ignore
            } else {
                return Err(Error::new_spanned(m, "unexpected attribute"));
            }
        }
    }

    if !is_c_repr {
        return Err(Error::new(Span::call_site(), "struct has to be #[repr(C)]"));
    }

    Ok(packed)
}

fn generate_packed_wrapper(
    vis: &Visibility,
    ident: &Ident,
    packed: usize,
) -> (TokenStream2, TokenStream2) {
    let alignment_ident = format_ident!("Volatile{}Alignment", ident);
    let wrapper_ident = format_ident!("Volatile{}AlignmentWrapper", ident);
    let lit = Literal::usize_unsuffixed(packed);

    let defs = quote! {
        #vis struct #alignment_ident<T>(T);

        unsafe impl<P> ::zst_volatile::alignment::Alignment for #alignment_ident<P>
        where
            P: ::zst_volatile::alignment::Alignment,
        {
            type Wrapper<T> = #wrapper_ident<P::Wrapper<T>>;

            #[inline]
            fn wrap<T>(value: T) -> Self::Wrapper<T> {
                #wrapper_ident(P::wrap(value))
            }

            #[inline]
            fn unwrap<T>(value: Self::Wrapper<T>) -> T {
                P::unwrap(value.0)
            }
        }

        #[repr(C, packed(#lit))]
        #vis struct #wrapper_ident<T>(T);
    };
    let ty = quote! { #alignment_ident::<A> };
    (defs, ty)
}
