use super::{TokenStreamPair, TypeDefinitionExtensions};
use proc_macro2::TokenStream;
use witx::{Id, IntDatatype};

impl TypeDefinitionExtensions for IntDatatype {
    fn get_type_definitions(&self, _ident: &Id, _docs: TokenStream) -> TokenStreamPair {
        unimplemented!("Type::Int")
    }

    fn has_rust_value(&self) -> bool {
        false
    }
}
