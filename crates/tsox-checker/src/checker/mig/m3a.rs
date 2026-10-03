#![allow(unused_imports)]

use crate::checker::types::*;
use crate::checker::types_type_id::*;
use crate::checker::utilities_is_private_within_ambient::value_to_string;
use std::sync::Arc;
use tsox_frontend::ast::{Node, Symbol};

pub fn format_type_flags(flags: TypeFlags) -> Vec<&'static str> { ::tsox_core::fntrace::enter("format_type_flags"); 
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

pub fn type_flags_to_string(flags: TypeFlags) -> String { ::tsox_core::fntrace::enter("type_flags_to_string"); 
    format_type_flags(flags).join("|")
}

pub fn variance_flags_to_string(v: VarianceFlags) -> String { ::tsox_core::fntrace::enter("variance_flags_to_string"); 
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
    pub fn id(&self) -> TypeId { ::tsox_core::fntrace::enter("id"); 
        self.id
    }

    pub fn object_flags(&self) -> ObjectFlags { ::tsox_core::fntrace::enter("object_flags"); 
        self.object_flags
    }

    pub fn symbol(&self) -> Option<&Arc<Symbol>> { ::tsox_core::fntrace::enter("symbol"); 
        self.symbol.as_ref()
    }

    pub fn target_interface_type(&self) -> Option<&InterfaceTypeData> { ::tsox_core::fntrace::enter("target_interface_type"); 
        let target = self.as_object()?.target.as_ref()?;
        match &target.data {
            TypeData::Interface(i) => Some(i),
            TypeData::Tuple(t) => Some(&t.interface_data),
            _ => None,
        }
    }

    pub fn target_tuple_type(&self) -> Option<&TupleTypeData> { ::tsox_core::fntrace::enter("target_tuple_type"); 
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
    pub fn value(&self) -> &LiteralValue { ::tsox_core::fntrace::enter("value"); 
        &self.value
    }

    pub fn fresh_type(&self) -> Option<&Arc<Type>> { ::tsox_core::fntrace::enter("fresh_type"); 
        self.fresh_type.get()
    }

    pub fn regular_type(&self) -> Option<&Arc<Type>> { ::tsox_core::fntrace::enter("regular_type"); 
        self.regular_type.get()
    }

    pub fn string(&self) -> String { ::tsox_core::fntrace::enter("string"); 
        value_to_string(&self.value)
    }
}

impl TupleTypeData {
    pub fn is_readonly(&self) -> bool { ::tsox_core::fntrace::enter("is_readonly"); 
        self.readonly
    }
}

impl TypeParameterData {
    pub fn is_this_type(&self) -> bool { ::tsox_core::fntrace::enter("is_this_type"); 
        self.is_this_type
    }
}

impl TemplateLiteralTypeData {
    pub fn texts(&self) -> &[String] { ::tsox_core::fntrace::enter("texts"); 
        &self.texts
    }
}

impl SubstitutionTypeData {
    pub fn subst_constraint(&self) -> Option<&Arc<Type>> { ::tsox_core::fntrace::enter("subst_constraint"); 
        self.constraint.as_ref()
    }
}

impl TypeAlias {
    pub fn symbol(&self) -> Option<&Arc<Symbol>> { ::tsox_core::fntrace::enter("symbol"); 
        self.symbol.as_ref()
    }

    pub fn type_arguments(&self) -> &[Arc<Type>] { ::tsox_core::fntrace::enter("type_arguments"); 
        &self.type_arguments
    }
}

impl Signature {
    pub fn id(&self) -> SignatureId { ::tsox_core::fntrace::enter("id"); 
        self.id
    }

    pub fn parameters(&self) -> &[Arc<Symbol>] { ::tsox_core::fntrace::enter("parameters"); 
        &self.parameters
    }

    pub fn this_parameter(&self) -> Option<&Arc<Symbol>> { ::tsox_core::fntrace::enter("this_parameter"); 
        self.this_parameter.as_ref()
    }
}

impl StructuredTypeData {
    pub fn properties(&self) -> &[Arc<Symbol>] { ::tsox_core::fntrace::enter("properties"); 
        &self.properties
    }
}

impl TypePredicate {
    pub fn kind(&self) -> TypePredicateKind { ::tsox_core::fntrace::enter("kind"); 
        self.kind
    }

    pub fn parameter_index(&self) -> i32 { ::tsox_core::fntrace::enter("parameter_index"); 
        self.parameter_index
    }

    pub fn parameter_name(&self) -> &str { ::tsox_core::fntrace::enter("parameter_name"); 
        &self.parameter_name
    }

    pub fn ty(&self) -> Option<&Arc<Type>> { ::tsox_core::fntrace::enter("ty"); 
        self.t.as_ref()
    }
}

impl TupleElementInfo {
    pub fn labeled_declaration(&self) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("labeled_declaration"); 
        self.labeled_declaration.as_ref()
    }

    pub fn tuple_element_flags(&self) -> ElementFlags { ::tsox_core::fntrace::enter("tuple_element_flags"); 
        self.flags
    }
}

impl IndexedAccessTypeData {
    pub fn object_type(&self) -> Option<&Arc<Type>> { ::tsox_core::fntrace::enter("object_type"); 
        self.object_type.as_ref()
    }

    pub fn index_type(&self) -> Option<&Arc<Type>> { ::tsox_core::fntrace::enter("index_type"); 
        self.index_type.as_ref()
    }
}

impl IndexInfo {
    pub fn is_readonly(&self) -> bool { ::tsox_core::fntrace::enter("is_readonly"); 
        self.is_readonly
    }

    pub fn key_type(&self) -> Option<&Arc<Type>> { ::tsox_core::fntrace::enter("key_type"); 
        self.key_type.as_ref()
    }

    pub fn value_type(&self) -> Option<&Arc<Type>> { ::tsox_core::fntrace::enter("value_type"); 
        self.value_type.as_ref()
    }
}
