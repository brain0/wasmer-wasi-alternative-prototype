use super::{TokenStreamPair, TypeDefinitionExtensions};
use proc_macro2::TokenStream;
use witx::{Id, TypeRef};

pub(crate) struct ArrayRef<'a>(pub(crate) &'a TypeRef);

impl<'a> TypeDefinitionExtensions for ArrayRef<'a> {
    fn get_type_definitions(&self, _ident: &Id, _docs: TokenStream) -> TokenStreamPair {
        //eprintln!("FIXME: Skipping array definition for {}.", ident.as_str());

        TokenStreamPair::new()
    }

    fn has_rust_value(&self) -> bool {
        false
    }
}
