use super::{builtin_type_ext::BuiltinTypeExt, ToIdent};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::rc::Rc;
use witx::{BuiltinType, NamedType, Type, TypeRef};

pub(crate) trait TypeRefExt {
    fn to_type(&self) -> TokenStream;
    fn get_type_definition(&self) -> &Rc<Type>;
    fn is_string(&self) -> bool;
    fn as_array(&self) -> Option<&Rc<NamedType>>;
}

impl TypeRefExt for TypeRef {
    fn to_type(&self) -> TokenStream {
        match self {
            TypeRef::Name(ref named_type) => {
                named_type.name.to_ident_native(None).to_token_stream()
            }
            TypeRef::Value(ref value) => match **value {
                Type::Builtin(builtin) => builtin.to_inner(),
                Type::Pointer(ref pointee) | Type::ConstPointer(ref pointee) => {
                    let pointee_name = pointee.to_type();
                    quote! { witx_gen::WasmSlicePtr<#pointee_name> }
                }
                _ => panic!("Could not determine type name for {:?}.", self),
            },
        }
    }

    fn get_type_definition(&self) -> &Rc<Type> {
        match self {
            TypeRef::Name(ref named_type) => named_type.tref.get_type_definition(),
            TypeRef::Value(ref tp) => tp,
        }
    }

    fn is_string(&self) -> bool {
        if let TypeRef::Value(ref tp) = self {
            if let Type::Builtin(ref builtin_type) = **tp {
                if let BuiltinType::String = builtin_type {
                    return true;
                }
            }
        }

        false
    }

    fn as_array(&self) -> Option<&Rc<NamedType>> {
        if let witx::TypeRef::Name(ref named) = self {
            if let witx::TypeRef::Value(ref tp) = named.tref {
                if let witx::Type::Array(ref inner) = **tp {
                    if let witx::TypeRef::Name(ref inner_named) = inner {
                        return Some(inner_named);
                    }
                }
            }
        }

        None
    }
}
