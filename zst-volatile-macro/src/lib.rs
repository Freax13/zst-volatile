use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Error};

#[proc_macro_derive(VolatileStruct)]
pub fn derive_volatile(tts: TokenStream) -> TokenStream {
    // FIXME: Check for repr(C), currently we just assume this.

    let input = parse_macro_input!(tts as DeriveInput);

    let packed = parse_repr(&input.attrs).unwrap();

    let Data::Struct(strukt) = &input.data else {
        return Error::new_spanned(&input,"only structs are supported").into_compile_error().into();
    };

    let vis = &input.vis;
    let ident = &input.ident;
    let volatile_ident = format_ident!("Volatile{}", ident);

    let mut offset = quote! { ::zst_volatile::offset::Zero };
    let fields = strukt.fields.iter().map(|field| {
        let vis = &field.vis;
        // FIXME: Make this compatible with unnamed structs.
        let ident = &field.ident.as_ref().unwrap();
        let ty = &field.ty;

        if !packed {
            // Align the field.
            offset = quote! {
                ::zst_volatile::offset::Align::<#offset, #ty>
            };
        }

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

fn parse_repr(attrs: &[Attribute]) -> Result<bool, &'static str> {
    let repr = attrs
        .iter()
        .find(|a| a.path.is_ident("repr"))
        .ok_or("VolatileStruct can only apply to #[repr(C, ...)]")?;

    let repr_metas = match repr.parse_meta().unwrap() {
        syn::Meta::List(l) => l,
        _ => return Err("#[repr(...)] invalid format"),
    };

    // Make sure #[repr(C)] exits
    repr_metas
        .nested
        .iter()
        .find(|m| match m {
            syn::NestedMeta::Meta(syn::Meta::Path(p)) => p.is_ident("C"),
            _ => false,
        })
        .ok_or("VolatileStruct can only apply to #[repr(C, ...)]")?;

    // Check if packed struct
    Ok(repr_metas
        .nested
        .iter()
        .find(|m| match m {
            syn::NestedMeta::Meta(syn::Meta::Path(p)) => p.is_ident("packed"),
            _ => false,
        })
        .is_some())
}
