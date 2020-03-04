use super::{
    int_repr_ext::IntReprExt, StringExt, ToIdent, TokenStreamPair, TypeDefinitionExtensions,
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::collections::HashMap;
use syn::LitInt;
use witx::{Id, Layout, Type, TypeRef, UnionDatatype};

impl TypeDefinitionExtensions for UnionDatatype {
    fn get_type_definitions(&self, ident: &Id, docs: TokenStream) -> TokenStreamPair {
        let tags: HashMap<_, _>;
        let tag_repr;

        match self.tag.tref {
            TypeRef::Name(_) => panic!("Expected an enum, got {:?}", self.tag.tref),
            TypeRef::Value(ref tp) => match **tp {
                Type::Enum(ref enum_datatype) => {
                    tags = enum_datatype
                        .variants
                        .iter()
                        .enumerate()
                        .map(|(i, v)| (v.name.clone(), i))
                        .collect();
                    tag_repr = enum_datatype.repr.to_type();
                }
                _ => panic!("Expected an enum, got {:?}", tp),
            },
        };

        let variants = self.variants.iter().map(|v| {
            let docs = v.docs.as_docs();
            assert!(tags.contains_key(&v.name));
            let variant_ident = v.name.to_ident_native(None);
            let typename = match v.tref {
                Some(ref tref) => match tref {
                    TypeRef::Name(ref named_type) => {
                        let ident = named_type.name.to_ident_native(None);
                        quote! { #ident }
                    }
                    TypeRef::Value(_) => unimplemented!(),
                },
                None => quote! { () },
            };
            quote! {
                #docs
                #variant_ident(#typename)
            }
        });

        let ident_native = ident.to_ident_native(None);

        let default_trait_impl = if self.has_rust_value() {
            let variant_ident = self.variants[0].name.to_ident_native(None);

            quote! {
                impl Default for #ident_native {
                    fn default() -> Self {
                        Self::#variant_ident(Default::default())
                    }
                }
            }
        } else {
            TokenStream::new()
        };

        let layout = self.mem_size_align();

        let array_offset = (layout.size + (layout.align - layout.size % layout.align)) as u32;
        assert!(array_offset % (layout.align as u32) == 0);

        let union_layout = self.union_layout();
        let contents_offset = union_layout.contents_offset as u32;

        let read_impl = self.variants.iter().map(|v| {
            let tag = LitInt::new(&format!("{}", tags[&v.name]), Span::call_site());
            let variant_ident = v.name.to_ident_native(None);

            quote! { #tag => Self::#variant_ident(witx_gen::WasmValue::read(memory, offset + #contents_offset)) }
        });

        let write_impl = self.variants.iter().map(|v| {
            let tag = LitInt::new(&format!("{}", tags[&v.name]), Span::call_site());
            let variant_ident = v.name.to_ident_native(None);

            quote! {
                Self::#variant_ident(value) => {
                    let tag: #tag_repr = #tag;

                    tag.write(memory, offset);
                    value.write(memory, offset + #contents_offset);
                }
            }
        });

        let native = quote! {
            #docs
            #[derive(Debug, Copy, Clone)]
            pub enum #ident_native {
                #( #variants, )*
                /// Variant for unknown enum tags received from a WASI binary.
                Unknown(Private),
            }

            #default_trait_impl

            impl witx_gen::WasmValue for #ident_native {
                const ARRAY_OFFSET: u32 = #array_offset;

                fn read(memory: &witx_gen::reexports::Memory, offset: u32) -> Self {
                    match <#tag_repr as witx_gen::WasmValue>::read(memory, offset) {
                       #( #read_impl, )*
                       _ => Self::Unknown(Private(())),
                    }
                 }

                fn write(self, memory: &witx_gen::reexports::Memory, offset: u32) {
                    match self {
                        #( #write_impl )*
                        _ => panic!("Tried to write an invalid union value to WASM memory."),
                    }
                }
            }
        };

        let mapped = if self.has_rust_value() {
            let variants = self.variants.iter().map(|v| {
                let docs = v.docs.as_docs();
                let variant_ident = v.name.to_ident();
                let variant_type = v
                    .tref
                    .as_ref()
                    .map(|tref| match tref {
                        TypeRef::Name(ref named_type) => {
                            named_type.name.to_ident().to_token_stream()
                        }
                        _ => panic!("{}: Expected a named type, got {:?}", ident.as_str(), tref),
                    })
                    .unwrap_or_else(|| quote! { () });

                quote! {
                    #docs
                    #variant_ident(#variant_type)
                }
            });

            let variant_conversion = self.variants.iter().map(|v| {
                let variant_ident_native = v.name.to_ident_native(None);
                let variant_ident = v.name.to_ident();

                quote! { Self::NativeType::#variant_ident_native(v) => Self::#variant_ident(witx_gen::try_from_native!(native, v)) }
            });

            let variant_conversion_back = self.variants.iter().map(|v| {
                let variant_ident_native = v.name.to_ident_native(None);
                let variant_ident = v.name.to_ident();

                quote! { Self::#variant_ident(v) => Self::NativeType::#variant_ident_native(v.to_native()) }
            });

            let ident = ident.to_ident();

            quote! {
                #docs
                #[derive(Debug, Copy, Clone)]
                pub enum #ident {
                    #( #variants ),*
                }

                impl witx_gen::WasiValue for #ident {
                    type NativeType = self::native::#ident_native;

                    fn from_native(native: Self::NativeType) -> Result<Self, witx_gen::WasiValueError<Self>> {
                        Ok(match native {
                            #( #variant_conversion, )*
                            Self::NativeType::Unknown(_) => Err(witx_gen::WasiValueError::from_unknown(native))?,
                        })
                    }

                    fn to_native(self) -> Self::NativeType {
                        match self {
                            #( #variant_conversion_back, )*
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
        self.tag.has_rust_value()
            && self
                .variants
                .iter()
                .all(|v| v.tref.as_ref().map(|v| v.has_rust_value()).unwrap_or(true))
    }
}
