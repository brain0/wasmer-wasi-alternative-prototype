use super::{
    int_repr_ext::IntReprExt, to_ident::ToIdent, StringExt, TokenStreamPair,
    TypeDefinitionExtensions,
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::LitInt;
use witx::{EnumDatatype, Id};

impl TypeDefinitionExtensions for EnumDatatype {
    fn get_type_definitions(&self, ident: &Id, docs: TokenStream) -> TokenStreamPair {
        let repr = self.repr.to_type();

        let ident_native = ident.to_ident_native(None);

        let variants_native = self.variants.iter().enumerate().map(|(i, v)| {
            let docs = v.docs.as_docs();
            let ident = v.name.to_ident_native(Some(ident.as_str()));
            let value = LitInt::new(&format!("{}", i), Span::call_site());

            quote! {
                #docs
                pub const #ident: #ident_native = #value;
            }
        });

        let native = quote! {
            #docs
            pub type #ident_native = #repr;

            #( #variants_native )*
        };

        let variants = self.variants.iter().map(|v| {
            let docs = v.docs.as_docs();

            let ident = v.name.to_ident();

            quote! {
                #docs
                #ident
            }
        });

        let variants_back = self.variants.iter().map(|v| {
            let variant_ident = v.name.to_ident();
            let variant_ident_native = v.name.to_ident_native(Some(ident.as_str()));

            quote! {
                self::native::#variant_ident_native => Ok(Self::#variant_ident),
            }
        });
        let ident = ident.to_ident();

        let mapped = quote! {
            #docs
            #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
            #[non_exhaustive]
            pub enum #ident {
                #( #variants ),*
            }

            impl witx_gen::WasiValue for #ident {
                type NativeType = self::native::#ident_native;

                fn from_native(native: Self::NativeType) -> Result<Self, witx_gen::WasiValueError<Self>> {
                    match native {
                        #( #variants_back )*
                        _ => Err(witx_gen::WasiValueError::from_native(native)),
                    }
                }

                fn to_native(self) -> Self::NativeType {
                    self as #repr
                }
            }
        };

        TokenStreamPair::from_streams(native, mapped)
    }

    fn has_rust_value(&self) -> bool {
        true
    }
}
