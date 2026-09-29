#![allow(unused_imports)]

use crate::checker::types::*;
use crate::checker::types_type_id::*;
use crate::checker::utilities_is_private_within_ambient::value_to_string;
use std::sync::Arc;
use tsox_frontend::ast::{Node, Symbol};

pub fn format_type_flags(flags: TypeFlags) -> Vec<&'static str> {
    const TYPE_FLAG_NAMES: [(TypeFlags, &str); 28] = [
        (TypeFlags::Any, "Any"),
        (TypeFlags::Unknown, "Unknown"),
        (TypeFlags::Undefined, "Undefined"),
        (TypeFlags::Null, "Null"),
        (TypeFlags::Void, "Void"),
        (TypeFlags::String, "String"),
        (TypeFlags::Number, "Number"),
        (TypeFlags::BigInt, "BigInt"),
        (TypeFlags::Boolean, "Boolean"),
        (TypeFlags::ESSymbol, "ESSymbol"),
        (TypeFlags::StringLiteral, "StringLiteral"),
        (TypeFlags::NumberLiteral, "NumberLiteral"),
        (TypeFlags::BigIntLiteral, "BigIntLiteral"),
        (TypeFlags::BooleanLiteral, "BooleanLiteral"),
        (TypeFlags::UniqueESSymbol, "UniqueESSymbol"),
        (TypeFlags::EnumLiteral, "EnumLiteral"),
        (TypeFlags::Enum, "Enum"),
        (TypeFlags::NonPrimitive, "NonPrimitive"),
        (TypeFlags::Never, "Never"),
        (TypeFlags::TypeParameter, "TypeParameter"),
        (TypeFlags::Object, "Object"),
        (TypeFlags::Index, "Index"),
        (TypeFlags::TemplateLiteral, "TemplateLiteral"),
        (TypeFlags::StringMapping, "StringMapping"),
        (TypeFlags::Substitution, "Substitution"),
        (TypeFlags::IndexedAccess, "IndexedAccess"),
        (TypeFlags::Conditional, "Conditional"),
        (TypeFlags::Union, "Union"),
    ];
    let mut result = Vec::with_capacity(flags.bits().count_ones() as usize);
    for (flag, name) in TYPE_FLAG_NAMES {
        if flags.intersects(flag) {
            result.push(name);
        }
    }
    if result.is_empty() {
        result.push("None");
    }
    result
}

pub fn type_flags_to_string(flags: TypeFlags) -> String {
    format_type_flags(flags).join("|")
}

pub fn variance_flags_to_string(v: VarianceFlags) -> String {
    let variance = VARIANCE_FLAGS_VARIANCE_MASK.intersection(v);
    let mut result = if variance == VARIANCE_FLAGS_INVARIANT {
        "in out"
    } else if variance == VARIANCE_FLAGS_BIVARIANT {
        "[bivariant]"
    } else if variance == VarianceFlags::Contravariant {
        "in"
    } else if variance == VarianceFlags::Covariant {
        "out"
    } else if variance == VarianceFlags::Independent {
        "[independent]"
    } else {
        ""
    }
    .to_string();
    if v.intersects(VarianceFlags::Unmeasurable) {
        result.push_str(" (unmeasurable)");
    } else if v.intersects(VarianceFlags::Unreliable) {
        result.push_str(" (unreliable)");
    }
    result
}

impl Type {
    pub fn id(&self) -> TypeId {
        self.id
    }

    pub fn object_flags(&self) -> ObjectFlags {
        self.object_flags
    }

    pub fn symbol(&self) -> Option<&Arc<Symbol>> {
        self.symbol.as_ref()
    }

    pub fn target_interface_type(&self) -> Option<&InterfaceTypeData> {
        let target = self.as_object()?.target.as_ref()?;
        match &target.data {
            TypeData::Interface(i) => Some(i),
            TypeData::Tuple(t) => Some(&t.interface_data),
            _ => None,
        }
    }

    pub fn target_tuple_type(&self) -> Option<&TupleTypeData> {
        if let TypeData::Tuple(t) = &self.data {
            return Some(t);
        }
        let target = self.as_object()?.target.as_ref()?;
        match &target.data {
            TypeData::Tuple(t) => Some(t),
            _ => None,
        }
    }
}

impl LiteralTypeData {
    pub fn value(&self) -> &LiteralValue {
        &self.value
    }

    pub fn fresh_type(&self) -> Option<&Arc<Type>> {
        self.fresh_type.get()
    }

    pub fn regular_type(&self) -> Option<&Arc<Type>> {
        self.regular_type.get()
    }

    pub fn string(&self) -> String {
        value_to_string(&self.value)
    }
}

impl TupleTypeData {
    pub fn is_readonly(&self) -> bool {
        self.readonly
    }
}

impl TypeParameterData {
    pub fn is_this_type(&self) -> bool {
        self.is_this_type
    }
}

impl TemplateLiteralTypeData {
    pub fn texts(&self) -> &[String] {
        &self.texts
    }
}

impl SubstitutionTypeData {
    pub fn subst_constraint(&self) -> Option<&Arc<Type>> {
        self.constraint.as_ref()
    }
}

impl TypeAlias {
    pub fn symbol(&self) -> Option<&Arc<Symbol>> {
        self.symbol.as_ref()
    }

    pub fn type_arguments(&self) -> &[Arc<Type>] {
        &self.type_arguments
    }
}

impl Signature {
    pub fn id(&self) -> SignatureId {
        self.id
    }

    pub fn parameters(&self) -> &[Arc<Symbol>] {
        &self.parameters
    }

    pub fn this_parameter(&self) -> Option<&Arc<Symbol>> {
        self.this_parameter.as_ref()
    }
}

impl StructuredTypeData {
    pub fn properties(&self) -> &[Arc<Symbol>] {
        &self.properties
    }
}

impl TypePredicate {
    pub fn kind(&self) -> TypePredicateKind {
        self.kind
    }

    pub fn parameter_index(&self) -> i32 {
        self.parameter_index
    }

    pub fn parameter_name(&self) -> &str {
        &self.parameter_name
    }

    pub fn ty(&self) -> Option<&Arc<Type>> {
        self.t.as_ref()
    }
}

impl TupleElementInfo {
    pub fn labeled_declaration(&self) -> Option<&Arc<Node>> {
        self.labeled_declaration.as_ref()
    }

    pub fn tuple_element_flags(&self) -> ElementFlags {
        self.flags
    }
}

impl IndexedAccessTypeData {
    pub fn object_type(&self) -> Option<&Arc<Type>> {
        self.object_type.as_ref()
    }

    pub fn index_type(&self) -> Option<&Arc<Type>> {
        self.index_type.as_ref()
    }
}

impl IndexInfo {
    pub fn is_readonly(&self) -> bool {
        self.is_readonly
    }

    pub fn key_type(&self) -> Option<&Arc<Type>> {
        self.key_type.as_ref()
    }

    pub fn value_type(&self) -> Option<&Arc<Type>> {
        self.value_type.as_ref()
    }
}
