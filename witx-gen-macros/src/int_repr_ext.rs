use proc_macro2::TokenStream;
use quote::quote;
use witx::IntRepr;

pub(crate) trait IntReprExt {
    fn to_type(self) -> TokenStream;
}

impl IntReprExt for IntRepr {
    fn to_type(self) -> TokenStream {
        match self {
            IntRepr::U8 => quote! { u8 },
            IntRepr::U16 => quote! { u16 },
            IntRepr::U32 => quote! { u32 },
            IntRepr::U64 => quote! { u64 },
        }
    }
}
