use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Error};

#[proc_macro_derive(VolatileStruct)]
pub fn derive_volatile(tts: TokenStream) -> TokenStream {
    // FIXME: Check for repr(C), currently we just assume this.

    let input = parse_macro_input!(tts as DeriveInput);

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

        // Align the field.
        let field_offset = quote! {
            ::zst_volatile::offset::Align::<#offset, #ty>
        };

        // Calculate the minimum offset for the next field.
        offset = quote! {
            ::zst_volatile::offset::PastField::<#offset, #ty>
        };

        quote! {
            #vis #ident: ::zst_volatile::Volatile<#ty, #field_offset>
        }
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
