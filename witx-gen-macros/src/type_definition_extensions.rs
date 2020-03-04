use super::{
    array::ArrayRef, const_pointer::ConstPointerRef, pointer::PointerRef, TokenStreamPair,
};
use proc_macro2::TokenStream;
use witx::{Id, NamedType, Type, TypeRef};

pub(crate) trait TypeDefinitionExtensions {
    fn get_type_definitions(&self, ident: &Id, docs: TokenStream) -> TokenStreamPair;
    fn has_rust_value(&self) -> bool;
}

impl TypeDefinitionExtensions for TypeRef {
    fn get_type_definitions(&self, ident: &Id, docs: TokenStream) -> TokenStreamPair {
        match self {
            TypeRef::Name(ref name) => name.get_type_definitions(ident, docs),
            TypeRef::Value(ref value) => value.get_type_definitions(ident, docs),
        }
    }

    fn has_rust_value(&self) -> bool {
        match self {
            TypeRef::Name(ref name) => name.has_rust_value(),
            TypeRef::Value(ref value) => value.has_rust_value(),
        }
    }
}

impl TypeDefinitionExtensions for NamedType {
    fn get_type_definitions(&self, _ident: &Id, _docs: TokenStream) -> TokenStreamPair {
        unimplemented!("TypeRef::NamedType")
    }

    fn has_rust_value(&self) -> bool {
        self.tref.has_rust_value()
    }
}

impl TypeDefinitionExtensions for Type {
    fn get_type_definitions(&self, ident: &Id, docs: TokenStream) -> TokenStreamPair {
        match self {
            Type::Enum(ref enum_datatype) => enum_datatype.get_type_definitions(ident, docs),
            Type::Int(ref int_datatype) => int_datatype.get_type_definitions(ident, docs),
            Type::Flags(ref flags_datatype) => flags_datatype.get_type_definitions(ident, docs),
            Type::Struct(ref struct_datatype) => struct_datatype.get_type_definitions(ident, docs),
            Type::Union(ref union_datatype) => union_datatype.get_type_definitions(ident, docs),
            Type::Handle(ref handle_datatype) => handle_datatype.get_type_definitions(ident, docs),
            Type::Array(ref type_ref) => ArrayRef(type_ref).get_type_definitions(ident, docs),
            Type::Pointer(ref type_ref) => PointerRef(type_ref).get_type_definitions(ident, docs),
            Type::ConstPointer(ref type_ref) => {
                ConstPointerRef(type_ref).get_type_definitions(ident, docs)
            }
            Type::Builtin(builtin_type) => builtin_type.get_type_definitions(ident, docs),
        }
    }

    fn has_rust_value(&self) -> bool {
        match self {
            Type::Enum(ref enum_datatype) => enum_datatype.has_rust_value(),
            Type::Int(ref int_datatype) => int_datatype.has_rust_value(),
            Type::Flags(ref flags_datatype) => flags_datatype.has_rust_value(),
            Type::Struct(ref struct_datatype) => struct_datatype.has_rust_value(),
            Type::Union(ref union_datatype) => union_datatype.has_rust_value(),
            Type::Handle(ref handle_datatype) => handle_datatype.has_rust_value(),
            Type::Array(ref type_ref) => ArrayRef(type_ref).has_rust_value(),
            Type::Pointer(ref type_ref) => PointerRef(type_ref).has_rust_value(),
            Type::ConstPointer(ref type_ref) => ConstPointerRef(type_ref).has_rust_value(),
            Type::Builtin(builtin_type) => builtin_type.has_rust_value(),
        }
    }
}
