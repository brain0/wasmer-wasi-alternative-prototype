use super::{to_ident::ToIdent, TokenStreamPair, TypeDefinitionExtensions};
use proc_macro2::TokenStream;
use quote::quote;
use witx::{HandleDatatype, Id};

impl TypeDefinitionExtensions for HandleDatatype {
    fn get_type_definitions(&self, ident: &Id, docs: TokenStream) -> TokenStreamPair {
        // TODO: What is a "handle" exactly? We're treating it as an opaque u32 for now.
        let inner_type = quote! { u32 };

        let native_ident = ident.to_ident_native(None);

        let native = quote! {
            #docs
            pub type #native_ident = #inner_type;
        };

        let ident = ident.to_ident();
        let mapped = quote! {
            #docs
            #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct #ident(pub native::#native_ident);

            impl witx_gen::WasiValue for #ident {
                type NativeType = native::#native_ident;
                fn from_native(native: Self::NativeType) -> Result<Self, witx_gen::WasiValueError<Self>> {
                    Ok(Self(native))
                }
                fn to_native(self) -> Self::NativeType {
                    self.0
                }
            }
        };

        TokenStreamPair::from_streams(native, mapped)
    }

    fn has_rust_value(&self) -> bool {
        true
    }
}
