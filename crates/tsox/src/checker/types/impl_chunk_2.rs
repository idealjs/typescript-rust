#![allow(unused_imports)]

use super::*;

impl InterfaceTypeData {
    pub fn outer_type_parameters(&self) -> &[Arc<Type>] {
        if self.all_type_parameters.is_empty() {
            return &[];
        }
        &self.all_type_parameters[..self.outer_type_parameter_count]
    }

    pub fn local_type_parameters(&self) -> &[Arc<Type>] {
        if self.all_type_parameters.is_empty() {
            return &[];
        }
        let end = self.all_type_parameters.len().saturating_sub(1);
        &self.all_type_parameters[self.outer_type_parameter_count..end]
    }

    pub fn type_parameters(&self) -> &[Arc<Type>] {
        if self.all_type_parameters.is_empty() {
            return &[];
        }
        let end = self.all_type_parameters.len() - 1;
        &self.all_type_parameters[..end]
    }
}

#[derive(Debug, Default)]
pub struct TupleTypeData {
    pub interface_data: InterfaceTypeData,
    pub element_infos: Vec<TupleElementInfo>,
    pub min_length: usize,
    pub fixed_length: usize,
    pub combined_flags: ElementFlags,
    pub readonly: bool,
}

#[derive(Debug, Clone)]
pub struct TupleElementInfo {
    pub flags: ElementFlags,
    pub labeled_declaration: Option<Arc<Node>>,

    pub type_: Option<Arc<Type>>,
}

#[derive(Debug)]
pub struct MappedTypeData {
    pub object: ObjectTypeData,
    pub declaration: Option<Arc<Node>>,
    pub type_parameter: Option<Arc<Type>>,
    pub constraint_type: Option<Arc<Type>>,
    pub name_type: Option<Arc<Type>>,
    pub template_type: Option<Arc<Type>>,
    pub modifiers_type: Option<Arc<Type>>,
    pub resolved_apparent_type: OnceLock<Arc<Type>>,
    pub contains_error: bool,
}

#[derive(Debug)]
pub struct ReverseMappedTypeData {
    pub object: ObjectTypeData,
    pub source: Option<Arc<Type>>,
    pub mapped_type: Option<Arc<Type>>,
    pub constraint_type: Option<Arc<Type>>,
}

#[derive(Debug)]
pub struct EvolvingArrayTypeData {
    pub object: ObjectTypeData,
    pub element_type: Option<Arc<Type>>,
    pub final_array_type: OnceLock<Arc<Type>>,
}

#[derive(Debug)]
pub struct InstantiationExpressionTypeData {
    pub object: ObjectTypeData,
    pub node: Option<Arc<Node>>,
}

#[derive(Debug, Default)]
pub struct UnionOrIntersectionTypeData {
    pub structured: StructuredTypeData,
    pub types: Vec<Arc<Type>>,
}

#[derive(Debug, Default)]
pub struct UnionTypeData {
    pub union_or_intersection: UnionOrIntersectionTypeData,
    pub resolved_reduced_type: OnceLock<Arc<Type>>,
    pub regular_type: OnceLock<Arc<Type>>,
    pub origin: Option<Arc<Type>>,
    pub key_property_name: Option<String>,
    pub constituent_map: HashMap<TypeId, Arc<Type>>,
}

#[derive(Debug, Default)]
pub struct IntersectionTypeData {
    pub union_or_intersection: UnionOrIntersectionTypeData,
    pub resolved_apparent_type: OnceLock<Arc<Type>>,
    pub unique_literal_filled_instantiation: OnceLock<Arc<Type>>,
}

#[derive(Debug)]
pub struct TypeParameterData {
    pub constrained: ConstrainedTypeData,
    pub constraint: Option<Arc<Type>>,
    pub target: Option<Arc<Type>>,
    pub mapper: Option<Arc<TypeMapper>>,
    pub is_this_type: bool,
    pub resolved_default_type: OnceLock<Arc<Type>>,
}

#[derive(Debug)]
pub struct IndexTypeData {
    pub constrained: ConstrainedTypeData,
    pub target: Option<Arc<Type>>,
    pub index_flags: IndexFlags,
}

#[derive(Debug)]
pub struct IndexedAccessTypeData {
    pub constrained: ConstrainedTypeData,
    pub object_type: Option<Arc<Type>>,
    pub index_type: Option<Arc<Type>>,
    pub access_flags: AccessFlags,
}

#[derive(Debug)]
pub struct TemplateLiteralTypeData {
    pub constrained: ConstrainedTypeData,
    pub texts: Vec<String>,
    pub types: Vec<Arc<Type>>,
}

#[derive(Debug)]
pub struct StringMappingTypeData {
    pub constrained: ConstrainedTypeData,
    pub target: Option<Arc<Type>>,
}

#[derive(Debug)]
pub struct SubstitutionTypeData {
    pub constrained: ConstrainedTypeData,
    pub base_type: Option<Arc<Type>>,
    pub constraint: Option<Arc<Type>>,
}

#[derive(Debug)]
pub struct ConditionalRoot {
    pub node: Option<Arc<Node>>,
    pub check_type: Option<Arc<Type>>,
    pub extends_type: Option<Arc<Type>>,
    pub is_distributive: bool,

    pub check_type_parameter_symbol: Option<Arc<crate::ast::Symbol>>,
    pub infer_type_parameters: Vec<Arc<Type>>,
    pub outer_type_parameters: Vec<Arc<Type>>,
    pub alias: Option<Box<TypeAlias>>,

    pub creation_scopes: Vec<u64>,
}

#[derive(Debug)]
pub struct ConditionalTypeData {
    pub constrained: ConstrainedTypeData,
    pub root: Option<Box<ConditionalRoot>>,
    pub check_type: Option<Arc<Type>>,
    pub extends_type: Option<Arc<Type>>,
    pub resolved_true_type: OnceLock<Arc<Type>>,
    pub resolved_false_type: OnceLock<Arc<Type>>,
    pub resolved_inferred_true_type: OnceLock<Arc<Type>>,
    pub resolved_default_constraint: OnceLock<Arc<Type>>,
    pub resolved_constraint_of_distributive: OnceLock<Arc<Type>>,
    pub mapper: Option<Arc<TypeMapper>>,
    pub combined_mapper: Option<Arc<TypeMapper>>,

    pub creation_type_argument_stack: Vec<HashMap<usize, Arc<Type>>>,
}
