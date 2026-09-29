#![allow(unused_imports)]

use crate::checker::types::*;
use std::collections::HashMap;
use std::sync::Arc;

impl Type {
    pub fn as_intrinsic_type(&self) -> Option<&IntrinsicTypeData> {
        match &self.data {
            TypeData::Intrinsic(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_literal_type(&self) -> Option<&LiteralTypeData> {
        match &self.data {
            TypeData::Literal(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_unique_es_symbol_type(&self) -> Option<&UniqueESSymbolTypeData> {
        match &self.data {
            TypeData::UniqueESSymbol(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_tuple_type(&self) -> Option<&TupleTypeData> {
        match &self.data {
            TypeData::Tuple(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_instantiation_expression_type(&self) -> Option<&InstantiationExpressionTypeData> {
        match &self.data {
            TypeData::InstantiationExpression(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_mapped_type(&self) -> Option<&MappedTypeData> {
        match &self.data {
            TypeData::Mapped(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_reverse_mapped_type(&self) -> Option<&ReverseMappedTypeData> {
        match &self.data {
            TypeData::ReverseMapped(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_evolving_array_type(&self) -> Option<&EvolvingArrayTypeData> {
        match &self.data {
            TypeData::EvolvingArray(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_type_parameter(&self) -> Option<&TypeParameterData> {
        match &self.data {
            TypeData::TypeParameter(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_union_type(&self) -> Option<&UnionTypeData> {
        match &self.data {
            TypeData::Union(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_intersection_type(&self) -> Option<&IntersectionTypeData> {
        match &self.data {
            TypeData::Intersection(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_index_type(&self) -> Option<&IndexTypeData> {
        match &self.data {
            TypeData::Index(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_indexed_access_type(&self) -> Option<&IndexedAccessTypeData> {
        match &self.data {
            TypeData::IndexedAccess(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_template_literal_type(&self) -> Option<&TemplateLiteralTypeData> {
        match &self.data {
            TypeData::TemplateLiteral(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_string_mapping_type(&self) -> Option<&StringMappingTypeData> {
        match &self.data {
            TypeData::StringMapping(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_substitution_type(&self) -> Option<&SubstitutionTypeData> {
        match &self.data {
            TypeData::Substitution(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_conditional_type(&self) -> Option<&ConditionalTypeData> {
        match &self.data {
            TypeData::Conditional(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_conditional_type_mut(&mut self) -> Option<&mut ConditionalTypeData> {
        match &mut self.data {
            TypeData::Conditional(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_constrained_type(&self) -> Option<&ConstrainedTypeData> {
        match &self.data {
            TypeData::TypeParameter(d) => Some(&d.constrained),
            TypeData::Index(d) => Some(&d.constrained),
            TypeData::IndexedAccess(d) => Some(&d.constrained),
            TypeData::TemplateLiteral(d) => Some(&d.constrained),
            TypeData::StringMapping(d) => Some(&d.constrained),
            TypeData::Substitution(d) => Some(&d.constrained),
            TypeData::Conditional(d) => Some(&d.constrained),
            TypeData::Object(d) => Some(&d.structured.constrained),
            TypeData::Interface(d) => Some(&d.object.structured.constrained),
            TypeData::Tuple(d) => Some(&d.interface_data.object.structured.constrained),
            TypeData::Mapped(d) => Some(&d.object.structured.constrained),
            TypeData::ReverseMapped(d) => Some(&d.object.structured.constrained),
            TypeData::EvolvingArray(d) => Some(&d.object.structured.constrained),
            TypeData::InstantiationExpression(d) => Some(&d.object.structured.constrained),
            TypeData::Union(d) => Some(&d.union_or_intersection.structured.constrained),
            TypeData::Intersection(d) => Some(&d.union_or_intersection.structured.constrained),
            _ => None,
        }
    }

    pub fn as_structured_type(&self) -> Option<&StructuredTypeData> {
        match &self.data {
            TypeData::Object(d) => Some(&d.structured),
            TypeData::Interface(d) => Some(&d.object.structured),
            TypeData::Tuple(d) => Some(&d.interface_data.object.structured),
            TypeData::Mapped(d) => Some(&d.object.structured),
            TypeData::ReverseMapped(d) => Some(&d.object.structured),
            TypeData::EvolvingArray(d) => Some(&d.object.structured),
            TypeData::InstantiationExpression(d) => Some(&d.object.structured),
            TypeData::Union(d) => Some(&d.union_or_intersection.structured),
            TypeData::Intersection(d) => Some(&d.union_or_intersection.structured),
            _ => None,
        }
    }

    pub fn as_object_type(&self) -> Option<&ObjectTypeData> {
        match &self.data {
            TypeData::Object(d) => Some(d),
            TypeData::Interface(d) => Some(&d.object),
            TypeData::Tuple(d) => Some(&d.interface_data.object),
            TypeData::Mapped(d) => Some(&d.object),
            TypeData::ReverseMapped(d) => Some(&d.object),
            TypeData::EvolvingArray(d) => Some(&d.object),
            TypeData::InstantiationExpression(d) => Some(&d.object),
            _ => None,
        }
    }

    pub fn as_type_reference(&self) -> Option<&ObjectTypeData> {
        self.as_object_type()
    }

    pub fn as_interface_type(&self) -> Option<&InterfaceTypeData> {
        match &self.data {
            TypeData::Interface(d) => Some(d),
            TypeData::Tuple(d) => Some(&d.interface_data),
            _ => None,
        }
    }

    pub fn as_union_or_intersection_type(&self) -> Option<&UnionOrIntersectionTypeData> {
        match &self.data {
            TypeData::Union(d) => Some(&d.union_or_intersection),
            TypeData::Intersection(d) => Some(&d.union_or_intersection),
            _ => None,
        }
    }
}
