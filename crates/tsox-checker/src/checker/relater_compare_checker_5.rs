#![allow(unused_imports)]

use crate::checker::relater_compare::*;

impl Checker {
    pub fn instantiate_type_predicate(
        &mut self,
        _predicate: &TypePredicate,
        _mapper: &Arc<TypeMapper>,
    ) -> Option<Box<TypePredicate>> {
        None
    }

    pub fn new_type_predicate(
        &mut self,
        kind: TypePredicateKind,
        parameter_name: String,
        parameter_index: i32,
        t: Arc<Type>,
    ) -> Box<TypePredicate> {
        Box::new(TypePredicate {
            kind,
            parameter_name,
            parameter_index,
            t: Some(t),
        })
    }

    pub fn is_resolving_return_type_of_signature(&mut self, _signature: &Arc<Signature>) -> bool {
        false
    }

    pub fn find_matching_signatures(
        &mut self,
        _signature_lists: &[Vec<Arc<Signature>>],
        _signature: &Arc<Signature>,
        _list_index: usize,
    ) -> Vec<Arc<Signature>> {
        Vec::new()
    }

    pub fn is_matching_signature(
        &mut self,
        source: &Arc<Signature>,
        target: &Arc<Signature>,
        partial_match: bool,
    ) -> bool {
        self.compare_signatures_identical(source, target, partial_match, false, false)
            != Ternary::False
    }

    pub fn compare_type_predicates_identical(
        &mut self,
        source: &TypePredicate,
        target: &TypePredicate,
        _compare_types: &dyn Fn(&Arc<Type>, &Arc<Type>) -> Ternary,
    ) -> Ternary {
        if source.kind != target.kind {
            return Ternary::False;
        }
        if source.parameter_name != target.parameter_name {
            return Ternary::False;
        }
        Ternary::True
    }

    pub fn get_effective_constraint_of_intersection(
        &mut self,
        _types: &[Arc<Type>],
        _target_is_union: bool,
    ) -> Option<Arc<Type>> {
        None
    }

    pub fn template_literal_types_definitely_unrelated(
        &mut self,
        _source: &TemplateLiteralTypeData,
        _target: &TemplateLiteralTypeData,
    ) -> bool {
        false
    }

    pub fn is_type_matched_by_template_literal_type(
        &mut self,
        _source: &Arc<Type>,
        _target: &TemplateLiteralTypeData,
        _compare_types: TypeComparer,
    ) -> bool {
        false
    }

    pub fn infer_types_from_template_literal_type(
        &mut self,
        _source: &Arc<Type>,
        _target: &TemplateLiteralTypeData,
    ) -> Vec<Arc<Type>> {
        Vec::new()
    }

    pub fn get_string_like_type_for_type(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if t.flags.intersects(TYPE_FLAGS_STRING_LIKE) {
            Some(Arc::clone(t))
        } else {
            None
        }
    }

    pub fn is_valid_type_for_template_literal_placeholder(
        &mut self,
        _source: &Arc<Type>,
        _target: &Arc<Type>,
        _compare_types: TypeComparer,
    ) -> bool {
        false
    }

    pub fn is_member_of_string_mapping(
        &mut self,
        _source: &Arc<Type>,
        _target: &Arc<Type>,
    ) -> bool {
        false
    }

    pub fn apply_target_string_mapping_to_source(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> (Arc<Type>, Arc<Type>) {
        (Arc::clone(source), Arc::clone(target))
    }

    pub fn get_type_of_property_in_types(
        &mut self,
        _types: &[Arc<Type>],
        _name: &str,
    ) -> Option<Arc<Type>> {
        None
    }

    pub fn get_type_of_property_in_type(
        &mut self,
        _t: &Arc<Type>,
        _name: &str,
    ) -> Option<Arc<Type>> {
        None
    }

    pub fn is_type_subset_of_union(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        self.is_type_subset_of(source, target)
    }

    pub fn is_type_derived_from(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        if source.flags.contains(TypeFlags::Union) {
            return source
                .types()
                .is_some_and(|ts| ts.iter().all(|t| self.is_type_derived_from(t, target)));
        }
        if target.flags.contains(TypeFlags::Union) {
            return source
                .types()
                .is_some_and(|ts| ts.iter().any(|t| self.is_type_derived_from(source, t)));
        }
        if source.flags.contains(TypeFlags::Intersection) {
            return source
                .types()
                .is_some_and(|ts| ts.iter().any(|t| self.is_type_derived_from(t, target)));
        }
        if source
            .flags
            .intersects(crate::checker::types_type_flags_instantiable_non_primitive::TYPE_FLAGS_INSTANTIABLE_NON_PRIMITIVE)
        {
            let constraint = self
                .get_base_constraint_of_type(source)
                .unwrap_or_else(|| self.get_unknown_type());
            return self.is_type_derived_from(&constraint, target);
        }
        if self.is_empty_anonymous_object_type(target) {
            return source
                .flags
                .intersects(TypeFlags::Object | TypeFlags::NonPrimitive);
        }
        if let Some(global_object) = self.get_global_type_by_name("Object")
            && Arc::ptr_eq(target, &global_object)
        {
            return source.flags.intersects(TypeFlags::Object | TypeFlags::NonPrimitive)
                && !self.is_empty_anonymous_object_type(source);
        }
        if let Some(global_function) = self.get_global_type_by_name("Function")
            && Arc::ptr_eq(target, &global_function)
        {
            return source.flags.contains(TypeFlags::Object)
                && self.is_function_object_type(source);
        }
        let target = target.target().cloned().unwrap_or_else(|| Arc::clone(target));
        if self.has_base_type(source, &target) {
            return true;
        }
        if !self.is_array_type(&target)
            || target
                .object_flags
                .contains(crate::checker::types::ObjectFlags::IsReadonlyArray)
        {
            return false;
        }
        match self.global_readonly_array_type.get().cloned() {
            Some(ro) => self.is_type_derived_from(source, &ro),
            None => false,
        }
    }

    // Go hasBaseType：名义基类型链。本仓类/接口实例型是携带声明符号的
    // Anonymous 结构型（无 Class/Reference 位），恒等按符号 Arc 判等，
    // 基链沿声明 heritage 递归
    fn has_base_type(&mut self, t: &Arc<Type>, check_base: &Arc<Type>) -> bool {
        if Arc::ptr_eq(t, check_base) || t.id == check_base.id {
            return true;
        }
        if let (Some(s1), Some(s2)) = (t.symbol.as_ref(), check_base.symbol.as_ref())
            && s1.flags.intersects(tsox_frontend::ast::SymbolFlags::Class)
            && s2.flags.intersects(tsox_frontend::ast::SymbolFlags::Class)
            && Arc::ptr_eq(s1, s2)
        {
            return true;
        }
        if t.flags.contains(TypeFlags::Intersection) {
            return t
                .types()
                .is_some_and(|ts| ts.iter().any(|x| self.has_base_type(x, check_base)));
        }
        let Some(sym) = t.symbol.clone() else {
            return false;
        };
        let mut heritage: Vec<Arc<Node>> = Vec::new();
        for decl in sym.declarations.iter() {
            match &decl.data {
                tsox_frontend::ast::NodeData::ClassDeclaration(d) => {
                    if let Some(clauses) = &d.heritage_clauses {
                        for c in clauses.iter() {
                            if let tsox_frontend::ast::NodeData::HeritageClause(hc) = &c.data {
                                for h in hc.types.iter() {
                                    if let tsox_frontend::ast::NodeData::ExpressionWithTypeArguments(ed) = &h.data {
                                        heritage.push(Arc::clone(&ed.expression));
                                    }
                                }
                            }
                        }
                    }
                }
                tsox_frontend::ast::NodeData::InterfaceDeclaration(d) => {
                    if let Some(clauses) = &d.heritage_clauses {
                        for c in clauses.iter() {
                            if let tsox_frontend::ast::NodeData::HeritageClause(hc) = &c.data {
                                for h in hc.types.iter() {
                                    match &h.data {
                                        tsox_frontend::ast::NodeData::TypeReferenceNode(tr) => {
                                            heritage.push(Arc::clone(&tr.type_name));
                                        }
                                        tsox_frontend::ast::NodeData::ExpressionWithTypeArguments(ed) => {
                                            heritage.push(Arc::clone(&ed.expression));
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
                _ => continue,
            }
        }
        for e in heritage {
            let bt = self.get_type_of_node(&e);
            let bt = self.get_instance_type_of_constructor(&bt).unwrap_or(bt);
            if self.has_base_type(&bt, check_base) {
                return true;
            }
        }
        false
    }

    fn is_function_object_type(&mut self, t: &Arc<Type>) -> bool {
        if let Some(sym) = self.get_global_type_by_name("Function")
            && let Some(target) = sym.target().cloned()
        {
            return self.has_base_type(t, &target) || Arc::ptr_eq(t, &sym);
        }
        false
    }

    pub fn is_distribution_dependent(&mut self, _root: &ConditionalRoot) -> bool {
        false
    }
}
