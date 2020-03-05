use super::{
    int_repr_ext::IntReprExt, to_ident::ToIdent, StringExt, TokenStreamPair,
    TypeDefinitionExtensions,
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::LitInt;
use witx::{FlagsDatatype, Id};

impl TypeDefinitionExtensions for FlagsDatatype {
    fn get_type_definitions(&self, ident: &Id, docs: TokenStream) -> TokenStreamPair {
        let repr = self.repr.to_type();

        let ident_native = ident.to_ident_native(None);

        let variants_native = self.flags.iter().enumerate().map(|(i, f)| {
            let docs = f.docs.as_docs();
            let ident = f.name.to_ident_native(Some(ident.as_str()));
            let value = LitInt::new(&format!("{}", i), Span::call_site());

            quote! {
                #docs
                pub const #ident: #ident_native = 1 << #value;
            }
        });

        let native = quote! {
            #docs
            pub type #ident_native = #repr;

            #( #variants_native )*
        };

        let consts = self.flags.iter().map(|f| {
            let docs = f.docs.as_docs();
            let flag_ident = f.name.to_ident();
            let flag_ident_native = f.name.to_ident_native(Some(ident.as_str()));

            quote! {
                #docs
                const #flag_ident = native::#flag_ident_native;
            }
        });

        let ident = ident.to_ident();

        let mapped = quote! {
            witx_gen::reexports::bitflags! {
                #docs
                pub struct #ident : #repr {
                    #( #consts )*
                }
            }

            impl witx_gen::WasiValue for #ident {
                type NativeType = #repr;

                fn from_native(native: Self::NativeType) -> std::result::Result<Self, witx_gen::WasiValueError<Self>> {
                    Self::from_bits(native).ok_or(witx_gen::WasiValueError::from_native(native))
                }

                fn to_native(self) -> Self::NativeType {
                    self.bits()
                }
            }
        };

        TokenStreamPair::from_streams(native, mapped)
    }

    fn has_rust_value(&self) -> bool {
        true
    }
}
