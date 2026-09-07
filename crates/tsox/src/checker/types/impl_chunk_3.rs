#![allow(unused_imports)]

use super::*;

impl Type {
    pub fn new(flags: TypeFlags, data: TypeData) -> Self {
        Self {
            flags,
            object_flags: ObjectFlags::empty(),
            id: next_type_id(),
            symbol: None,
            alias: None,
            data,
        }
    }

    pub fn is_union(&self) -> bool {
        self.flags.contains(TypeFlags::Union)
    }

    pub fn is_intersection(&self) -> bool {
        self.flags.contains(TypeFlags::Intersection)
    }

    pub fn is_string(&self) -> bool {
        self.flags.contains(TypeFlags::String)
    }

    pub fn is_string_literal(&self) -> bool {
        self.flags.contains(TypeFlags::StringLiteral)
    }

    pub fn is_number_literal(&self) -> bool {
        self.flags.contains(TypeFlags::NumberLiteral)
    }

    pub fn is_big_int_literal(&self) -> bool {
        self.flags.contains(TypeFlags::BigIntLiteral)
    }

    pub fn is_enum_literal(&self) -> bool {
        self.flags.contains(TypeFlags::EnumLiteral)
    }

    pub fn is_boolean_like(&self) -> bool {
        self.flags.intersects(TYPE_FLAGS_BOOLEAN_LIKE)
    }

    pub fn is_string_like(&self) -> bool {
        self.flags.intersects(TYPE_FLAGS_STRING_LIKE)
    }

    pub fn is_class(&self) -> bool {
        self.object_flags.contains(ObjectFlags::Class)
    }

    pub fn is_type_parameter(&self) -> bool {
        self.flags.contains(TypeFlags::TypeParameter)
    }

    pub fn is_index(&self) -> bool {
        self.flags.contains(TypeFlags::Index)
    }

    pub fn distributed(&self) -> Vec<Arc<Type>> {
        if self.flags.contains(TypeFlags::Union) {
            if let TypeData::Union(u) = &self.data {
                return u.union_or_intersection.types.clone();
            }
        }
        if self.flags.contains(TypeFlags::Never) {
            return Vec::new();
        }

        Vec::new()
    }

    pub fn target(&self) -> Option<&Arc<Type>> {
        match &self.data {
            TypeData::Object(o) => o.target.as_ref(),
            TypeData::Interface(i) => i.object.target.as_ref(),
            TypeData::Tuple(t) => t.interface_data.object.target.as_ref(),
            TypeData::Mapped(m) => m.object.target.as_ref(),
            TypeData::ReverseMapped(r) => r.object.target.as_ref(),
            TypeData::EvolvingArray(e) => e.object.target.as_ref(),
            TypeData::InstantiationExpression(i) => i.object.target.as_ref(),
            TypeData::TypeParameter(t) => t.target.as_ref(),
            TypeData::Index(i) => i.target.as_ref(),
            TypeData::StringMapping(s) => s.target.as_ref(),
            _ => None,
        }
    }

    pub fn mapper(&self) -> Option<&Arc<TypeMapper>> {
        match &self.data {
            TypeData::Object(o) => o.mapper.as_ref(),
            TypeData::Interface(i) => i.object.mapper.as_ref(),
            TypeData::Tuple(t) => t.interface_data.object.mapper.as_ref(),
            TypeData::Mapped(m) => m.object.mapper.as_ref(),
            TypeData::ReverseMapped(r) => r.object.mapper.as_ref(),
            TypeData::EvolvingArray(e) => e.object.mapper.as_ref(),
            TypeData::InstantiationExpression(i) => i.object.mapper.as_ref(),
            TypeData::TypeParameter(t) => t.mapper.as_ref(),
            TypeData::Conditional(c) => c.mapper.as_ref(),
            _ => None,
        }
    }

    pub fn types(&self) -> Option<&[Arc<Type>]> {
        match &self.data {
            TypeData::Union(u) => Some(&u.union_or_intersection.types),
            TypeData::Intersection(i) => Some(&i.union_or_intersection.types),
            TypeData::TemplateLiteral(t) => Some(&t.types),
            _ => None,
        }
    }

    pub fn as_structured(&self) -> Option<&StructuredTypeData> {
        match &self.data {
            TypeData::Object(o) => Some(&o.structured),
            TypeData::Interface(i) => Some(&i.object.structured),
            TypeData::Tuple(t) => Some(&t.interface_data.object.structured),
            TypeData::Mapped(m) => Some(&m.object.structured),
            TypeData::ReverseMapped(r) => Some(&r.object.structured),
            TypeData::EvolvingArray(e) => Some(&e.object.structured),
            TypeData::InstantiationExpression(i) => Some(&i.object.structured),
            TypeData::Union(u) => Some(&u.union_or_intersection.structured),
            TypeData::Intersection(i) => Some(&i.union_or_intersection.structured),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&ObjectTypeData> {
        match &self.data {
            TypeData::Object(o) => Some(o),
            TypeData::Interface(i) => Some(&i.object),
            TypeData::Tuple(t) => Some(&t.interface_data.object),
            TypeData::Mapped(m) => Some(&m.object),
            TypeData::ReverseMapped(r) => Some(&r.object),
            TypeData::EvolvingArray(e) => Some(&e.object),
            TypeData::InstantiationExpression(i) => Some(&i.object),
            _ => None,
        }
    }

    pub fn as_interface(&self) -> Option<&InterfaceTypeData> {
        match &self.data {
            TypeData::Interface(i) => Some(i),
            TypeData::Tuple(t) => Some(&t.interface_data),
            _ => None,
        }
    }

    pub fn as_union_or_intersection(&self) -> Option<&UnionOrIntersectionTypeData> {
        match &self.data {
            TypeData::Union(u) => Some(&u.union_or_intersection),
            TypeData::Intersection(i) => Some(&i.union_or_intersection),
            _ => None,
        }
    }

    pub fn intrinsic_name(&self) -> Option<&str> {
        if let TypeData::Intrinsic(i) = &self.data {
            Some(&i.intrinsic_name)
        } else {
            None
        }
    }

    pub fn literal_value(&self) -> Option<&LiteralValue> {
        if let TypeData::Literal(l) = &self.data {
            Some(&l.value)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Signature {
    pub id: SignatureId,
    pub flags: SignatureFlags,
    pub min_argument_count: i32,
    pub resolved_min_argument_count: i32,
    pub declaration: Option<Arc<Node>>,
    pub type_parameters: Vec<Arc<Type>>,
    pub parameters: Vec<Arc<Symbol>>,
    pub this_parameter: Option<Arc<Symbol>>,
    pub resolved_return_type: OnceLock<Arc<Type>>,
    pub resolved_type_predicate: Option<Box<TypePredicate>>,
    pub target: Option<Arc<Signature>>,
    pub mapper: Option<Arc<TypeMapper>>,
    pub isolated_signature_type: OnceLock<Arc<Type>>,

    pub instantiated_parameter_types: Option<Vec<Arc<Type>>>,
}

impl Signature {
    pub fn new() -> Self {
        Self {
            id: 0,
            flags: SignatureFlags::None,
            min_argument_count: 0,
            resolved_min_argument_count: 0,
            declaration: None,
            type_parameters: Vec::new(),
            parameters: Vec::new(),
            this_parameter: None,
            resolved_return_type: OnceLock::new(),
            resolved_type_predicate: None,
            target: None,
            mapper: None,
            isolated_signature_type: OnceLock::new(),
            instantiated_parameter_types: None,
        }
    }

    pub fn has_rest_parameter(&self) -> bool {
        self.flags.contains(SignatureFlags::HasRestParameter)
    }

    pub fn min_argument_count(&self) -> usize {
        self.min_argument_count as usize
    }
}

impl Default for Signature {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct CompositeSignature {
    pub is_union: bool,
    pub signatures: Vec<Arc<Signature>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TypePredicateKind {
    #[default]
    This,
    Identifier,
    AssertsThis,
    AssertsIdentifier,
}

#[derive(Debug, Clone)]
pub struct TypePredicate {
    pub kind: TypePredicateKind,
    pub parameter_index: i32,
    pub parameter_name: String,
    pub t: Option<Arc<Type>>,
}

#[derive(Debug)]
pub struct IndexInfo {
    pub key_type: Option<Arc<Type>>,
    pub value_type: Option<Arc<Type>>,
    pub is_readonly: bool,
    pub declaration: Option<Arc<Node>>,
    pub index_symbol: Option<Arc<Symbol>>,
    pub components: Vec<Arc<Node>>,
}

#[derive(Debug, Default)]
pub struct SymbolReferenceLinks {
    pub reference_kinds: crate::ast::SymbolFlags,
}

#[derive(Debug, Default)]
pub struct ValueSymbolLinks {
    pub resolved_type: Option<Arc<Type>>,
    pub write_type: Option<Arc<Type>>,
    pub target: Option<Arc<Symbol>>,
    pub mapper: Option<Arc<TypeMapper>>,
    pub name_type: Option<Arc<Type>>,
    pub containing_type: Option<Arc<Type>>,
    pub function_or_constructor_checked: bool,
}

#[derive(Debug, Default)]
pub struct MappedSymbolLinks {
    pub key_type: Option<Arc<Type>>,
    pub synthetic_origin: Option<Arc<Symbol>>,
}

#[derive(Debug, Default)]
pub struct DeferredSymbolLinks {
    pub parent: Option<Arc<Type>>,
    pub constituents: Vec<Arc<Type>>,
    pub write_constituents: Vec<Arc<Type>>,
}
