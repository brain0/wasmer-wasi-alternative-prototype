use super::{TokenStreamPair, TypeDefinitionExtensions};
use proc_macro2::TokenStream;
use witx::{Id, TypeRef};

pub(crate) struct PointerRef<'a>(pub(crate) &'a TypeRef);

impl<'a> TypeDefinitionExtensions for PointerRef<'a> {
    fn get_type_definitions(&self, _ident: &Id, _docs: TokenStream) -> TokenStreamPair {
        unimplemented!("Type::Pointer")
    }

    fn has_rust_value(&self) -> bool {
        false
    }
}
