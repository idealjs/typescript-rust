#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn is_type_parameter_at_top_level_in_return_type(
        &self,
        signature: &Arc<Signature>,
        type_parameter: &Type,
    ) -> bool {
        if let Some(return_type) = signature.resolved_return_type.get() {
            return self.is_type_parameter_at_top_level(return_type, type_parameter, 0);
        }
        false
    }

    pub(crate) fn get_type_from_inference(&self, inference: &InferenceInfo) -> Option<Arc<Type>> {
        if !inference.candidates.is_empty() {
            Some(self.create_union_type(inference.candidates.clone()))
        } else if !inference.contra_candidates.is_empty() {
            Some(self.create_intersection_type(inference.contra_candidates.clone()))
        } else {
            None
        }
    }

    pub(crate) fn get_common_supertype(&mut self, types: &[Arc<Type>]) -> Arc<Type> {
        if types.len() == 1 {
            return types[0].clone();
        }
        let primary_types: Vec<Arc<Type>> = types.to_vec();
        if self.literal_types_with_same_base_type(&primary_types) {
            return self.create_union_type(primary_types);
        }
        let supertype = self.get_single_common_supertype(&primary_types);
        let nullable_flags = self.get_combined_type_flags(types) & TYPE_FLAGS_NULLABLE;
        if nullable_flags != TypeFlags::None {
            self.get_nullable_type(&supertype, nullable_flags)
        } else {
            supertype
        }
    }

    pub(crate) fn get_single_common_supertype(&mut self, types: &[Arc<Type>]) -> Arc<Type> {
        let candidate = self.find_leftmost_type(types);

        let all_are_strict_subtypes = types
            .iter()
            .all(|t| Arc::ptr_eq(t, &candidate) || self.is_type_strict_subtype_of(t, &candidate));
        if all_are_strict_subtypes {
            return candidate;
        }

        let mut candidate: Option<Arc<Type>> = None;
        for t in types {
            match &candidate {
                None => candidate = Some(t.clone()),
                Some(c) => {
                    if self.is_type_subtype_of(c, t) {
                        candidate = Some(t.clone());
                    }
                }
            }
        }
        candidate.unwrap_or_else(|| self.unknown_type())
    }

    pub(crate) fn find_leftmost_type(&mut self, types: &[Arc<Type>]) -> Arc<Type> {
        let mut candidate: Option<Arc<Type>> = None;
        for t in types {
            match &candidate {
                None => candidate = Some(t.clone()),
                Some(_c) => {
                    candidate = Some(t.clone());
                }
            }
        }
        candidate.unwrap_or_else(|| self.unknown_type())
    }

    pub(crate) fn get_common_subtype(&mut self, types: &[Arc<Type>]) -> Arc<Type> {
        let mut subtype: Option<Arc<Type>> = None;
        for t in types {
            match &subtype {
                None => subtype = Some(t.clone()),
                Some(s) => {
                    if self.is_type_subtype_of(t, s) {
                        subtype = Some(t.clone());
                    }
                }
            }
        }
        subtype.unwrap_or_else(|| self.unknown_type())
    }

    pub(crate) fn get_combined_type_flags(&self, types: &[Arc<Type>]) -> TypeFlags {
        let mut flags = TypeFlags::None;
        for t in types {
            if t.flags.contains(TypeFlags::Union) {
                if let Some(inner_types) = t.types() {
                    flags |= self.get_combined_type_flags(inner_types);
                }
            } else {
                flags |= t.flags;
            }
        }
        flags
    }

    pub(crate) fn literal_types_with_same_base_type(&self, types: &[Arc<Type>]) -> bool {
        let mut common_base_type: Option<Arc<Type>> = None;
        for t in types {
            if t.flags.contains(TypeFlags::Never) {
                continue;
            }
            let base_type = self.get_base_type_of_literal_type(t);
            match &common_base_type {
                None => common_base_type = Some(base_type.clone()),
                Some(cbt) => {
                    if Arc::ptr_eq(&base_type, t) || !Arc::ptr_eq(&base_type, cbt) {
                        return false;
                    }
                }
            }
        }
        true
    }

    pub(crate) fn is_const_type_variable(&self, _t: &Type, _depth: i32) -> bool {
        false
    }

    pub(crate) fn get_default_constraint_of_conditional_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let Some(constraint) = self.get_constraint_of_type_parameter(t) {
            return Some(constraint);
        }
        None
    }

    pub(crate) fn maybe_type_of_kind(&self, t: &Type, flags: TypeFlags) -> bool {
        t.flags.intersects(flags)
    }

    pub(crate) fn create_union_type(&self, types: Vec<Arc<Type>>) -> Arc<Type> {
        let filtered: Vec<Arc<Type>> = types
            .into_iter()
            .filter(|t| !t.flags.contains(TypeFlags::Never))
            .collect();
        if filtered.is_empty() {
            return self.never_type();
        }
        if filtered.len() == 1 {
            return filtered[0].clone();
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: filtered,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: HashMap::new(),
            }),
        ))
    }

    pub(crate) fn create_intersection_type(&self, types: Vec<Arc<Type>>) -> Arc<Type> {
        if types.is_empty() {
            return self.unknown_type();
        }
        if types.len() == 1 {
            return types[0].clone();
        }
        Arc::new(Type::new(
            TypeFlags::Intersection,
            TypeData::Intersection(IntersectionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types,
                },
                resolved_apparent_type: std::sync::OnceLock::new(),
                unique_literal_filled_instantiation: std::sync::OnceLock::new(),
            }),
        ))
    }

    #[allow(dead_code)]
    pub(crate) fn get_this_argument_type(&self, _node: &crate::ast::Node) -> Arc<Type> {
        self.undefined_type()
    }

    #[allow(dead_code)]
    pub(crate) fn get_spread_argument_type(
        &self,
        _args: &[Arc<crate::ast::Node>],
        _start: usize,
        _end: usize,
    ) -> Arc<Type> {
        self.unknown_type()
    }

    pub(crate) fn is_object_or_array_literal_type(&self, t: &Type) -> bool {
        t.flags.contains(TypeFlags::Object)
            && t.object_flags
                .intersects(ObjectFlags::ObjectLiteral | ObjectFlags::ArrayLiteral)
    }
}

pub(super) fn function_like_parameters(
    node: &Arc<crate::ast::Node>,
) -> Option<Arc<crate::ast::NodeList>> {
    use crate::ast::NodeData;
    match &node.data {
        NodeData::FunctionExpression(d) => Some(Arc::clone(&d.parameters)),
        NodeData::ArrowFunction(d) => Some(Arc::clone(&d.parameters)),
        NodeData::FunctionDeclaration(d) => Some(Arc::clone(&d.parameters)),
        NodeData::MethodDeclaration(d) => Some(Arc::clone(&d.parameters)),
        NodeData::MethodSignatureDeclaration(d) => Some(Arc::clone(&d.parameters)),
        NodeData::GetAccessorDeclaration(d) => Some(Arc::clone(&d.parameters)),
        NodeData::SetAccessorDeclaration(d) => Some(Arc::clone(&d.parameters)),
        _ => None,
    }
}

pub(crate) fn is_this_parameter_node(param: &Arc<crate::ast::Node>) -> bool {
    if let crate::ast::NodeData::ParameterDeclaration(pd) = &param.data {
        return matches!(&pd.name.data, crate::ast::NodeData::Identifier(id) if id.text == "this");
    }
    false
}
