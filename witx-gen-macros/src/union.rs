use super::{
    int_repr_ext::IntReprExt, to_ident::ToIdent, StringExt, TokenStreamPair,
    TypeDefinitionExtensions,
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
        let tag_repr_size;

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
                    tag_repr_size = enum_datatype.repr.mem_size_align().size;
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
                impl std::default::Default for #ident_native {
                    fn default() -> Self {
                        Self::#variant_ident(std::default::Default::default())
                    }
                }
            }
        } else {
            TokenStream::new()
        };

        let layout = self.mem_size_align();

        let size = LitInt::new(&format!("{}", layout.size), Span::call_site());
        let array_offset = (layout.size + (layout.align - layout.size % layout.align)) as u32;
        assert!(array_offset % (layout.align as u32) == 0);

        let union_layout = self.union_layout();

        let read_impl = self.variants.iter().map(|v| {
            let tag = LitInt::new(&format!("{}", tags[&v.name]), Span::call_site());
            let variant_ident = v.name.to_ident_native(None);

            if let Some(ref tref) = v.tref {
                let start = union_layout.contents_offset;
                let end = union_layout.contents_offset + tref.mem_size_align().size;

                quote! { #tag => Self::#variant_ident(witx_gen::WasmValue::read(&mem[#start..#end])) }
            } else {
                quote! { #tag => Self::#variant_ident(()) }
            }
        });

        let write_impl = self.variants.iter().map(|v| {
            let tag = LitInt::new(&format!("{}", tags[&v.name]), Span::call_site());
            let variant_ident = v.name.to_ident_native(None);

            if let Some(ref tref) = v.tref {
                let start = union_layout.contents_offset;
                let end = union_layout.contents_offset + tref.mem_size_align().size;

                quote! {
                    Self::#variant_ident(value) => {
                        let tag: #tag_repr = #tag;

                        tag.write(&mem[..#tag_repr_size]);
                        value.write(&mem[#start..#end]);
                    }
                }
            } else {
                quote! {
                        Self::#variant_ident(value) => {
                        let tag: #tag_repr = #tag;

                        tag.write(&mem[..#tag_repr_size]);
                    }
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
                const SIZE: u32 = #size;
                const ARRAY_OFFSET: u32 = #array_offset;

                fn read(mem: &[std::cell::Cell<u8>]) -> Self {
                    assert_eq!(mem.len(), #size);
                    match <#tag_repr as witx_gen::WasmValue>::read(&mem[..#tag_repr_size]) {
                        #( #read_impl, )*
                        _ => Self::Unknown(Private(())),
                     }
                }

                fn write(self, mem: &[std::cell::Cell<u8>]) {
                    assert_eq!(mem.len(), #size);
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

                    fn from_native(native: Self::NativeType) -> std::result::Result<Self, witx_gen::WasiValueError<Self>> {
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
