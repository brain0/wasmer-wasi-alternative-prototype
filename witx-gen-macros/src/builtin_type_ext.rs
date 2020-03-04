use proc_macro2::TokenStream;
use quote::quote;
use witx::BuiltinType;

pub(crate) trait BuiltinTypeExt {
    fn to_inner(self) -> TokenStream;
}

impl BuiltinTypeExt for BuiltinType {
    fn to_inner(self) -> TokenStream {
        match self {
            BuiltinType::String => unimplemented!("Strings not supported"),
            BuiltinType::Char8 => quote! { u8 },
            BuiltinType::USize => quote! { u32 },
            BuiltinType::U8 => quote! { u8 },
            BuiltinType::U16 => quote! { u16 },
            BuiltinType::U32 => quote! { u32 },
            BuiltinType::U64 => quote! { u64 },
            BuiltinType::S8 => quote! { i8 },
            BuiltinType::S16 => quote! { i16 },
            BuiltinType::S32 => quote! { i32 },
            BuiltinType::S64 => quote! { i64 },
            BuiltinType::F32 => quote! { f32 },
            BuiltinType::F64 => quote! { f64 },
        }
    }
}
