use super::{
    type_ref_ext::TypeRefExt, StringExt, ToIdent, TokenStreamPair, TypeDefinitionExtensions,
};
use proc_macro2::TokenStream;
use quote::quote;
use witx::{Id, Layout, StructDatatype, TypeRef};

impl TypeDefinitionExtensions for StructDatatype {
    fn get_type_definitions(&self, ident: &Id, docs: TokenStream) -> TokenStreamPair {
        let ident_native = ident.to_ident_native(None);

        let fields = self.members.iter().map(|m| {
            let docs = m.docs.as_docs();
            let field_ident = m.name.to_ident_native(None);
            let field_type = m.tref.to_type();

            quote! {
                #docs
                pub #field_ident: #field_type
            }
        });

        let member_layout = self.member_layout();

        let read_impl = member_layout.iter().map(|l| {
            let ident = l.member.name.to_ident_native(None);
            let offset = l.offset as u32;

            quote! {
                #ident: witx_gen::WasmValue::read(memory, offset + #offset)
            }
        });

        let write_impl = member_layout.iter().map(|l| {
            let ident = l.member.name.to_ident_native(None);
            let offset = l.offset as u32;

            quote! {
                witx_gen::WasmValue::write(self.#ident, memory, offset + #offset);
            }
        });

        let derived_traits = if self.has_rust_value() {
            quote! { #[derive(Debug, Copy, Clone, Default)] }
        } else {
            quote! { #[derive(Debug, Copy, Clone)] }
        };

        let layout = self.mem_size_align();

        let array_offset = (layout.size + (layout.align - layout.size % layout.align)) as u32;
        assert!(array_offset % (layout.align as u32) == 0);

        let native = quote! {
            #docs
            #derived_traits
            pub struct #ident_native {
                #( #fields ),*
            }

            impl witx_gen::WasmValue for #ident_native {
                const ARRAY_OFFSET: u32 = #array_offset;

                fn read(memory: &witx_gen::reexports::Memory, offset: u32) -> Self {
                    Self {
                        #( #read_impl ),*
                    }
                }

                fn write(self, memory: &witx_gen::reexports::Memory, offset: u32) {
                    #( #write_impl )*
                }
            }
        };

        let mapped = if self.has_rust_value() {
            let fields = self.members.iter().map(|m| {
                let docs = m.docs.as_docs();
                let field_ident = m.name.to_ident();
                let field_type = match m.tref {
                    TypeRef::Name(ref named_type) => named_type.name.to_ident(),
                    _ => panic!(
                        "{}: Expected a named type, got {:?}",
                        ident.as_str(),
                        m.tref
                    ),
                };

                quote! {
                    #docs
                    pub #field_ident: #field_type
                }
            });

            let field_conversion = self.members.iter().map(|m| {
                let field_ident_native = m.name.to_ident_native(None);
                let field_ident = m.name.to_ident();

                quote! { #field_ident: witx_gen::try_from_native!(native, native.#field_ident_native) }
            });

            let field_conversion_back = self.members.iter().map(|m| {
                let field_ident_native = m.name.to_ident_native(None);
                let field_ident = m.name.to_ident();

                quote! { #field_ident_native: self.#field_ident.to_native() }
            });

            let ident = ident.to_ident();

            quote! {
                #docs
                #[derive(Debug, Copy, Clone)]
                pub struct #ident {
                    #( #fields ),*
                }

                impl witx_gen::WasiValue for #ident {
                    type NativeType = self::native::#ident_native;

                    fn from_native(native: Self::NativeType) -> Result<Self, witx_gen::WasiValueError<Self>> {
                        Ok(Self {
                            #( #field_conversion, )*
                        })
                    }

                    fn to_native(self) -> Self::NativeType {
                        Self::NativeType {
                            #( #field_conversion_back, )*
                        }
                    }
                }
            }
        } else {
            TokenStream::new()
        };

        TokenStreamPair::from_streams(native, mapped)
    }

    fn has_rust_value(&self) -> bool {
        self.members.iter().all(|m| m.tref.has_rust_value())
    }
}
