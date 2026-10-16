#![allow(unused_imports)]

use crate::checker::inference::*;

impl Checker {
    // Go isTypeParameterAtTopLevelInReturnType（inference.go:1560-1568）：
    // 断言/守卫签名优先在谓词型上考察（asserts value is T 的 T 在谓词顶层），
    // 无谓词才回落 getReturnTypeOfSignature 强制解析（返回型注解/体推断惰性链
    // 就地展开）；返回型未驻留时 peek 恒 miss 判 false 使 widenLiteralTypes 误真
    pub(crate) fn is_type_parameter_at_top_level_in_return_type(
        &mut self,
        signature: &Arc<Signature>,
        type_parameter: &Type,
    ) -> bool { ::tsox_core::fntrace::enter("is_type_parameter_at_top_level_in_return_type");
        if let Some(type_predicate) = self.compute_type_predicate_of_signature(signature) {
            return type_predicate
                .t
                .is_some_and(|t| self.is_type_parameter_at_top_level(&t, type_parameter, 0));
        }
        let Some(return_type) = self.get_return_type_of_signature(signature) else {
            return false;
        };
        self.is_type_parameter_at_top_level(&return_type, type_parameter, 0)
    }

    pub(crate) fn get_type_from_inference(&mut self, inference: &InferenceInfo) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_type_from_inference"); 
        if !inference.candidates.is_empty() {
            Some(self.get_union_type_ex(
                inference.candidates.clone(),
                crate::checker::exports_union_reduction::UnionReduction::Subtype, None, None))
        } else if !inference.contra_candidates.is_empty() {
            Some(self.create_intersection_type(inference.contra_candidates.clone()))
        } else {
            None
        }
    }

    pub(crate) fn get_common_supertype(&mut self, types: &[Arc<Type>]) -> Arc<Type> { ::tsox_core::fntrace::enter("get_common_supertype"); 
        if types.len() == 1 {
            return types[0].clone();
        }
        let mut primary_types: Vec<Arc<Type>> = types.to_vec();
        if self.strict_null_checks {
            primary_types = primary_types
                .into_iter()
                .map(|t| {
                    self.filter_type(&t, &mut |u| {
                        !u.flags.intersects(TypeFlags::Null | TypeFlags::Undefined)
                    })
                })
                .collect();
        }
        let supertype = if self.literal_types_with_same_base_type(&primary_types) {
            self.get_union_type(primary_types.clone())
        } else {
            self.get_single_common_supertype(&primary_types)
        };
        let same = primary_types.len() == types.len()
            && primary_types
                .iter()
                .zip(types.iter())
                .all(|(a, b)| Arc::ptr_eq(a, b));
        if same {
            return supertype;
        }
        let nullable_flags =
            self.get_combined_type_flags(types) & (TypeFlags::Null | TypeFlags::Undefined);
        self.get_nullable_type(&supertype, nullable_flags)
    }

    pub(crate) fn get_single_common_supertype(&mut self, types: &[Arc<Type>]) -> Arc<Type> { ::tsox_core::fntrace::enter("get_single_common_supertype"); 
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

    pub(crate) fn find_leftmost_type(&mut self, types: &[Arc<Type>]) -> Arc<Type> { ::tsox_core::fntrace::enter("find_leftmost_type"); 
        let mut candidate: Option<Arc<Type>> = None;
        for t in types {
            match &candidate {
                None => candidate = Some(t.clone()),
                Some(c) => {
                    if self.is_type_strict_subtype_of(c, t) {
                        candidate = Some(t.clone());
                    }
                }
            }
        }
        candidate.unwrap_or_else(|| self.unknown_type())
    }

    pub(crate) fn get_common_subtype(&mut self, types: &[Arc<Type>]) -> Arc<Type> { ::tsox_core::fntrace::enter("get_common_subtype"); 
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

    pub(crate) fn get_combined_type_flags(&self, types: &[Arc<Type>]) -> TypeFlags { ::tsox_core::fntrace::enter("get_combined_type_flags"); 
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

    pub(crate) fn literal_types_with_same_base_type(&self, types: &[Arc<Type>]) -> bool { ::tsox_core::fntrace::enter("literal_types_with_same_base_type"); 
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

    pub(crate) fn is_const_type_variable(&self, _t: &Type, _depth: i32) -> bool { ::tsox_core::fntrace::enter("is_const_type_variable"); 
        false
    }

    pub(crate) fn get_default_constraint_of_conditional_type(
        &mut self,
        t: &Arc<Type>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_default_constraint_of_conditional_type");
        if let Some(cached) = t.resolved_default_constraint() {
            return Some(cached);
        }
        let true_constraint = self.get_inferred_true_type_of_conditional(t)?;
        let false_constraint = self.get_forced_branch_type_of_conditional_type(t, false)?;
        let constraint = if true_constraint.flags.contains(TypeFlags::Any) {
            false_constraint
        } else if false_constraint.flags.contains(TypeFlags::Any) {
            true_constraint
        } else {
            self.get_union_type(vec![true_constraint, false_constraint])
        };
        if let TypeData::Conditional(ct) = &t.data {
            let _ = ct.resolved_default_constraint.set(Arc::clone(&constraint));
        }
        Some(constraint)
    }

    pub(crate) fn maybe_type_of_kind(&self, t: &Type, flags: TypeFlags) -> bool { ::tsox_core::fntrace::enter("maybe_type_of_kind"); 
        if t.flags.intersects(flags) {
            return true;
        }
        if t
            .flags
            .intersects(TypeFlags::Union | TypeFlags::Intersection)
        {
            if let Some(ui) = t.as_union_or_intersection() {
                return ui.types.iter().any(|c| self.maybe_type_of_kind(c, flags));
            }
        }
        false
    }

    pub(crate) fn create_union_type(&self, types: Vec<Arc<Type>>) -> Arc<Type> { ::tsox_core::fntrace::enter("create_union_type"); 
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

    pub(crate) fn create_intersection_type(&self, types: Vec<Arc<Type>>) -> Arc<Type> { ::tsox_core::fntrace::enter("create_intersection_type"); 
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
                resolved_properties: std::sync::OnceLock::new(),
            }),
        ))
    }

    pub(crate) fn get_this_argument_type(
        &mut self,
        node: Option<&Arc<tsox_frontend::ast::Node>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_this_argument_type");
        // Go getThisArgumentType（checker.go:9891-9904）：非方法访问调用无 this
        // 实参按 voidType 参与 this 位推断；可选链根去 null，可选链去抹除标记
        let Some(node) = node else {
            return self.void_type();
        };
        let this_argument_type = self.check_expression_ex(
            node,
            crate::checker::types_alias_symbol_links::CheckMode::Normal,
        );
        if let Some(parent) = node.parent() {
            if tsox_frontend::ast::mig::m3g_2::is_optional_chain_root(&parent) {
                return self.get_non_nullable_type(&this_argument_type);
            }
            if tsox_frontend::ast::is_optional_chain(&parent) {
                return self.remove_optional_type_marker(&this_argument_type);
            }
        }
        this_argument_type
    }

    #[allow(dead_code)]
    pub(crate) fn get_spread_argument_type(
        &self,
        _args: &[Arc<tsox_frontend::ast::Node>],
        _start: usize,
        _end: usize,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_spread_argument_type"); 
        self.unknown_type()
    }

    pub(crate) fn is_object_or_array_literal_type(&self, t: &Type) -> bool { ::tsox_core::fntrace::enter("is_object_or_array_literal_type"); 
        t.flags.contains(TypeFlags::Object)
            && t.object_flags
                .intersects(ObjectFlags::ObjectLiteral | ObjectFlags::ArrayLiteral)
    }
}

pub(super) fn function_like_parameters(
    node: &Arc<tsox_frontend::ast::Node>,
) -> Option<Arc<tsox_frontend::ast::NodeList>> { ::tsox_core::fntrace::enter("function_like_parameters"); 
    use tsox_frontend::ast::NodeData;
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

pub(crate) fn is_this_parameter_node(param: &Arc<tsox_frontend::ast::Node>) -> bool { ::tsox_core::fntrace::enter("is_this_parameter_node"); 
    if let tsox_frontend::ast::NodeData::ParameterDeclaration(pd) = &param.data {
        return matches!(&pd.name.data, tsox_frontend::ast::NodeData::Identifier(id) if id.text == "this");
    }
    false
}
