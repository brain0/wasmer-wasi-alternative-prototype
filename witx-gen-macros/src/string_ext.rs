use proc_macro2::TokenStream;
use quote::quote;

pub(crate) trait StringExt {
    fn as_docs(&self) -> TokenStream;
}

impl StringExt for str {
    fn as_docs(&self) -> TokenStream {
        let lines = self.lines();
        quote! {
            #( #[doc = #lines] )*
        }
    }
}
