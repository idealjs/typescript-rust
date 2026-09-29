#![allow(unused_imports)]

use crate::checker::checker::*;
use crate::checker::checker_iteration::*;
use crate::checker::checker_this_container::get_this_parameter;
use crate::checker::mapper::{append_type_mapping, prepend_type_mapping};
use crate::checker::mig::m2c_3::some_type;
use crate::checker::mig::m2c::r18k3_defs::MappedTypeNameTypeKind;
use crate::checker::mig::m1f_2::IterationTypes;
use crate::checker::mig::w11a::substitution_base_type;
use crate::checker::mig::wc2::r21k3_defs::*;
use crate::checker::mig::wc2::r22k3_defs::*;
use crate::checker::mig::wc2::r23k3_defs::*;
use crate::checker::mig::wc2::r24k8_defs::map_type_ex_self;
use crate::checker::mig::wc2::{is_tuple_type, r20k3_defs};
use crate::checker::mig::wc3::MappedTypeModifiers;
use crate::checker::mig::wc3_3::is_generic_tuple_type;
use crate::checker::relater_recursion_identity::RecursionIdentity as RecursionId;
use crate::checker::types::*;
use crate::checker::utilities_is_optional_symbol::{is_fresh_literal_type, is_type_any};
use std::sync::Arc;
use tsox_frontend::ast::Diagnostic;
use tsox_frontend::ast::mig::m3b::{module_specifier as node_module_specifier, properties};
use tsox_frontend::ast::mig::m3f_2::node_parameters;
use tsox_frontend::ast::{self, Node, NodeData, Symbol, SyntaxKind};

impl Checker {
    pub fn get_iteration_types_of_iterable_fast(
        &mut self,
        t: &Arc<Type>,
        r: &IterationTypesResolver,
    ) -> IterationTypes {
        if [
            (r.get_global_iterable_type)(),
            (r.get_global_iterator_object_type)(),
            (r.get_global_iterable_iterator_type)(),
            (r.get_global_generator_type)(),
        ]
        .into_iter()
        .flatten()
        .any(|g| self.is_reference_to_type(t, &g))
        {
            let type_arguments = self.get_type_arguments(t);
            return r.get_resolved_iteration_types(
                &type_arguments[0],
                &type_arguments[1],
                Some(Arc::clone(&type_arguments[2])),
            );
        }
        if self.is_reference_to_some_type(t, &(r.get_global_builtin_iterator_types)()) {
            let type_arguments = self.get_type_arguments(t);
            return r.get_resolved_iteration_types(
                &type_arguments[0],
                &self.get_builtin_iterator_return_type(),
                Some(self.unknown_type()),
            );
        }
        IterationTypes::default()
    }

    pub fn get_iteration_types_of_iterable_slow(
        &mut self,
        t: &Arc<Type>,
        r: &IterationTypesResolver,
        error_node: Option<&Arc<Node>>,
        diagnostic_output: &mut Vec<Diagnostic>,
    ) -> IterationTypes {
        let method_name = self.get_property_name_for_known_symbol_name(&r.iterator_symbol_name);
        let method = self.get_property_of_type(t, &method_name);
        if let Some(method) = method {
            if !method.flags.intersects(SymbolFlags::Optional) {
                let method_type = self.get_type_of_symbol(&method);
                if is_type_any(&method_type) {
                    return IterationTypes {
                        yield_type: Some(self.any_type()),
                        return_type: Some(self.any_type()),
                        next_type: Some(self.any_type()),
                    };
                }
                let all_signatures = self.get_signatures_of_type(&method_type, SignatureKind::Call);
                let valid_signatures: Vec<Arc<Signature>> = all_signatures
                    .iter()
                    .filter(|sig| self.get_min_argument_count(sig) == 0)
                    .cloned()
                    .collect();
                if !valid_signatures.is_empty() {
                    let return_types: Vec<Arc<Type>> = valid_signatures
                        .iter()
                        .map(|sig| {
                            self.get_return_type_of_signature(sig)
                                .unwrap_or_else(|| self.unknown_type())
                        })
                        .collect();
                    let iterator_type = self.get_intersection_type(return_types);
                    return self.get_iteration_types_of_iterator_worker(&iterator_type, r, error_node, diagnostic_output);
                }
                if let Some(error_node) = error_node {
                    if !all_signatures.is_empty() {
                        if let Some(iterable_checked) = (r.get_global_iterable_type_checked)() {
                            self.check_type_assignable_to_ex(
                                t,
                                &iterable_checked,
                                Some(error_node),
                                None,
                                Some(diagnostic_output),
                            );
                        }
                    }
                }
            }
        }
        IterationTypes::default()
    }

    pub fn get_iteration_types_of_iterator_fast(
        &mut self,
        t: &Arc<Type>,
        r: &IterationTypesResolver,
    ) -> IterationTypes {
        if [
            (r.get_global_iterator_type)(),
            (r.get_global_iterator_object_type)(),
            (r.get_global_iterable_iterator_type)(),
            (r.get_global_generator_type)(),
        ]
        .into_iter()
        .flatten()
        .any(|g| self.is_reference_to_type(t, &g))
        {
            let type_arguments = self.get_type_arguments(t);
            return r.get_resolved_iteration_types(
                &type_arguments[0],
                &type_arguments[1],
                Some(Arc::clone(&type_arguments[2])),
            );
        }
        if self.is_reference_to_some_type(t, &(r.get_global_builtin_iterator_types)()) {
            let type_arguments = self.get_type_arguments(t);
            return r.get_resolved_iteration_types(
                &type_arguments[0],
                &self.get_builtin_iterator_return_type(),
                Some(self.unknown_type()),
            );
        }
        IterationTypes::default()
    }

    pub fn get_iteration_types_of_iterator_slow(
        &mut self,
        t: &Arc<Type>,
        r: &IterationTypesResolver,
        error_node: Option<&Arc<Node>>,
        diagnostic_output: &mut Vec<Diagnostic>,
    ) -> IterationTypes {
        let next_types = self.get_iteration_types_of_method(t, r, "next", error_node, diagnostic_output);
        let return_types = self.get_iteration_types_of_method(t, r, "return", error_node, diagnostic_output);
        let throw_types = self.get_iteration_types_of_method(t, r, "throw", error_node, diagnostic_output);
        self.combine_iteration_types_r20k3(vec![next_types, return_types, throw_types])
    }

    pub fn get_iteration_types_of_iterator_worker(
        &mut self,
        t: &Arc<Type>,
        r: &IterationTypesResolver,
        error_node: Option<&Arc<Node>>,
        diagnostic_output: &mut Vec<Diagnostic>,
    ) -> IterationTypes {
        if is_type_any(t) {
            return IterationTypes {
                yield_type: Some(self.any_type()),
                return_type: Some(self.any_type()),
                next_type: Some(self.any_type()),
            };
        }
        let iteration_types = self.get_iteration_types_of_iterator_fast(t, r);
        if iteration_types.yield_type.is_some() || iteration_types.return_type.is_some() || iteration_types.next_type.is_some() {
            return iteration_types;
        }
        self.get_iteration_types_of_iterator_slow(t, r, error_node, diagnostic_output)
    }

    pub fn get_late_bound_symbol(&mut self, symbol: &Arc<Symbol>) -> Arc<Symbol> {
        if !symbol.flags.intersects(SymbolFlags::CLASS_MEMBER)
            || symbol.name != internal_symbol_name_computed()
        {
            return Arc::clone(symbol);
        }
        let needs_init = self
            .late_bound_links
            .get(symbol)
            .map(|links| links.late_symbol.is_none())
            .unwrap_or(false)
            && symbol.declarations.iter().any(|d| self.has_late_bindable_name(d));
        if needs_init {
            let parent = symbol.parent().unwrap_or_else(|| Arc::clone(symbol));
            let parent = self.get_merged_symbol(&parent);
            if symbol.declarations.iter().any(|d| ast::has_static_modifier(d)) {
                self.get_exports_of_symbol(&parent);
            } else {
                self.get_members_of_symbol(&parent);
            }
        }
        if self
            .late_bound_links
            .get(symbol)
            .and_then(|l| l.late_symbol.clone())
            .is_none()
        {
            self.late_bound_links.get_or_default(symbol).late_symbol = Some(Arc::clone(symbol));
        }
        self.late_bound_links
            .get(symbol)
            .and_then(|l| l.late_symbol.clone())
            .unwrap_or_else(|| Arc::clone(symbol))
    }

    pub fn get_legacy_decorator_argument_count(&self, node: &Arc<Node>, signature: &Arc<Signature>) -> i32 {
        let Some(parent) = node.parent() else {
            panic!("Unhandled case in getLegacyDecoratorArgumentCount");
        };
        match parent.kind {
            SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => 1,
            SyntaxKind::PropertyDeclaration => {
                if ast::has_accessor_modifier(&parent) {
                    3
                } else {
                    2
                }
            }
            SyntaxKind::MethodDeclaration | SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => {
                if signature.parameters.len() <= 2 {
                    2
                } else {
                    3
                }
            }
            SyntaxKind::Parameter => 3,
            _ => panic!("Unhandled case in getLegacyDecoratorArgumentCount"),
        }
    }

    pub fn get_legacy_decorator_call_signature(&mut self, decorator: &Arc<Node>) -> Option<Arc<Signature>> {
        let Some(node) = decorator.parent() else {
            return None;
        };
        if self.signature_links.get(&node).map(|l| l.decorator_signature.is_some()).unwrap_or(false) {
            return self.decorator_signature_result(&node);
        }
        let any_sig = self.any_signature();
        self.signature_links.get_or_default(&node).decorator_signature = Some(any_sig);
        match node.kind {
            SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => {
                let target_type = self.get_type_of_symbol(&self.get_symbol_of_declaration(&node).unwrap_or_else(|| panic!("missing symbol")));
                let target_param = self.new_parameter("target", &target_type);
                let return_type = self.get_union_type(vec![Arc::clone(&target_type), self.void_type()]);
                let sig = self.new_call_signature(
                    None,
                    None,
                    vec![target_param],
                    Some(return_type),
                );
                self.signature_links.get_or_default(&node).decorator_signature = Some(sig);
            }
            SyntaxKind::Parameter => {
                let Some(parent) = node.parent() else {
                    return self.decorator_signature_result(&node);
                };
                let parent_grand = parent.parent();
                let ok = !ast::is_constructor_declaration(&parent)
                    && !(ast::is_method_declaration(&parent)
                        || (ast::is_set_accessor_declaration(&parent)
                            && parent_grand
                                .as_ref()
                                .map(|g| ast::is_class_like(g))
                                .unwrap_or(false)));
                let this_parameter = get_this_parameter(&parent);
                let not_node = this_parameter
                    .as_ref()
                    .map(|p| !Arc::ptr_eq(p, &node))
                    .unwrap_or(true);
                if ok && not_node {
                    let parameters = node_parameters(&parent)
                        .map(|l| l.nodes.as_slice())
                        .unwrap_or(&[]);
                    let index = parameters
                        .iter()
                        .position(|p| Arc::ptr_eq(p, &node))
                        .unwrap_or(0) as i32
                        - if this_parameter.is_some() { 1 } else { 0 };
                    let (target_type, key_type) = if ast::is_constructor_declaration(&parent) {
                        let container = parent.parent().unwrap_or_else(|| Arc::clone(&parent));
                        let target_type = self.get_type_of_symbol(&self.get_symbol_of_declaration(&container).unwrap_or_else(|| panic!("missing symbol")));
                        (target_type, self.undefined_type())
                    } else {
                        let target_type = self.get_parent_type_of_class_element(&parent);
                        let key_type = self.get_class_element_property_key_type(&parent);
                        (target_type, key_type)
                    };
                    let index_type = self.get_number_literal_type(tsox_core::jsnum::Number(index as f64));
                    let target_param = self.new_parameter("target", &target_type);
                    let key_param = self.new_parameter("propertyKey", &key_type);
                    let index_param = self.new_parameter("parameterIndex", &index_type);
                    let return_type = self.void_type();
                    let sig = self.new_call_signature(
                        None,
                        None,
                        vec![target_param, key_param, index_param],
                        Some(return_type),
                    );
                    self.signature_links.get_or_default(&node).decorator_signature = Some(sig);
                }
            }
            SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::PropertyDeclaration => {
                let Some(parent) = node.parent() else {
                    return self.decorator_signature_result(&node);
                };
                if !ast::is_class_like(&parent) {
                    return self.decorator_signature_result(&node);
                }
                let target_type = self.get_parent_type_of_class_element(&node);
                let target_param = self.new_parameter("target", &target_type);
                let key_type = self.get_class_element_property_key_type(&node);
                let key_param = self.new_parameter("propertyKey", &key_type);
                let return_type = if !ast::is_property_declaration(&node) {
                    let node_type = self.get_type_of_node(&node);
                    self.new_typed_property_descriptor_type(&node_type)
                } else {
                    self.void_type()
                };
                let has_prop_desc = !ast::is_property_declaration(&node) || ast::has_accessor_modifier(&node);
                if has_prop_desc {
                    let descriptor_type = self.get_type_of_node(&node);
                    let descriptor_param = self.new_parameter("descriptor", &descriptor_type);
                    let return_type = self.get_union_type(vec![Arc::clone(&return_type), self.void_type()]);
                    let sig = self.new_call_signature(
                        None,
                        None,
                        vec![target_param, key_param, descriptor_param],
                        Some(return_type),
                    );
                    self.signature_links.get_or_default(&node).decorator_signature = Some(sig);
                } else {
                    let return_type = self.get_union_type(vec![Arc::clone(&return_type), self.void_type()]);
                    let sig = self.new_call_signature(
                        None,
                        None,
                        vec![target_param, key_param],
                        Some(return_type),
                    );
                    self.signature_links.get_or_default(&node).decorator_signature = Some(sig);
                }
            }
            _ => {}
        }
        self.decorator_signature_result(&node)
    }

    fn decorator_signature_result(&self, node: &Arc<Node>) -> Option<Arc<Signature>> {
        let links = self.signature_links.get(node)?;
        match &links.decorator_signature {
            Some(sig) => {
                if Arc::ptr_eq(sig, &self.any_signature()) {
                    None
                } else {
                    Some(Arc::clone(sig))
                }
            }
            None => None,
        }
    }

    pub fn get_longest_candidate_index(&mut self, candidates: &[Arc<Signature>], args_count: i32) -> i32 {
        let mut max_params_index = -1;
        let mut max_params = -1;
        for (i, candidate) in candidates.iter().enumerate() {
            let param_count = self.get_parameter_count(candidate);
            if self.has_effective_rest_parameter(candidate) || param_count as i32 >= args_count {
                return i as i32;
            }
            if param_count as i32 > max_params {
                max_params = param_count as i32;
                max_params_index = i as i32;
            }
        }
        max_params_index
    }

    pub fn get_lower_bound_of_key_type(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.intersects(TypeFlags::Index) {
            let target = index_type_target(t);
            let apparent = self.get_apparent_type(&target);
            if is_generic_tuple_type(&apparent) {
                return self
                    .get_known_keys_of_tuple_type(&apparent)
                    .unwrap_or_else(|| self.unknown_type());
            }
            return self.get_index_type(&apparent);
        }
        if t.flags.intersects(TypeFlags::Conditional) {
            let root = conditional_root(t);
            if root.is_distributive {
                let check_type = conditional_check_type(t);
                let constraint = self.get_lower_bound_of_key_type(&check_type);
                if !Arc::ptr_eq(&constraint, &check_type) {
                    let mapping = prepend_type_mapping(
                        Arc::clone(root.check_type.as_ref().unwrap()),
                        Arc::clone(&constraint),
                        t.mapper().map(|m| m.as_ref()),
                    );
                    return self.get_conditional_type_instantiation(t, Some(&mapping), false, None);
                }
            }
            return Arc::clone(t);
        }
        if t.flags.intersects(TypeFlags::Union) {
            return map_type_ex_self(
                self,
                t,
                &mut |c, t| Some(c.get_lower_bound_of_key_type(t)),
                true,
            )
            .unwrap_or_else(|| Arc::clone(t));
        }
        if t.flags.intersects(TypeFlags::Intersection) {
            let types = t.types().unwrap_or(&[]).to_vec();
            if types.len() == 2
                && types[0]
                    .flags
                    .intersects(TypeFlags::String | TypeFlags::Number | TypeFlags::BigInt)
                && Arc::ptr_eq(&types[1], &self.empty_type_literal_type())
            {
                return Arc::clone(t);
            }
            let mapped: Vec<Arc<Type>> = types.iter().map(|u| self.get_lower_bound_of_key_type(u)).collect();
            return self.get_intersection_type(mapped);
        }
        Arc::clone(t)
    }

    pub fn get_mapped_type_name_type_kind(&mut self, t: &Arc<Type>) -> MappedTypeNameTypeKind {
        let Some(name_type) = self.get_name_type_from_mapped_type(t) else {
            return MappedTypeNameTypeKind::None;
        };
        let type_parameter = self
            .get_type_parameter_from_mapped_type(t)
            .unwrap_or_else(|| self.unknown_type());
        if self.is_type_assignable_to(&name_type, &type_parameter) {
            MappedTypeNameTypeKind::Rename
        } else {
            MappedTypeNameTypeKind::Remapping
        }
    }

    pub fn get_matching_union_constituent_for_object_literal(
        &mut self,
        union_type: &Arc<Type>,
        node: &Arc<Node>,
    ) -> Option<Arc<Type>> {
        let key_property_name = self.get_key_property_name(union_type);
        if key_property_name.as_deref().is_some_and(|k| !k.is_empty()) {
            let prop_node = properties(node).iter().find(|p| {
                let symbol_matches = self
                    .get_symbol_of_declaration(p)
                    .map(|s| Some(&s.name) == key_property_name.as_ref())
                    .unwrap_or(false);
                symbol_matches
                    && ast::is_property_assignment(p)
                    && p.initializer()
                        .map(|i| self.is_possibly_discriminant_value(i))
                        .unwrap_or(false)
            });
            if let Some(prop_node) = prop_node {
                let initializer = prop_node.initializer()?;
                let prop_type = self.get_context_free_type_of_expression(initializer);
                return self.get_constituent_type_for_key_type(union_type, &prop_type);
            }
        }
        None
    }

    pub fn get_module_specifier_for_import_or_export(&self, node: &Arc<Node>) -> Option<Arc<Node>> {
        fn parent_node(n: &Arc<Node>) -> Arc<Node> {
            n.parent().unwrap_or_else(|| Arc::clone(n))
        }
        match node.kind {
            SyntaxKind::ImportClause => get_module_specifier_from_node(&parent_node(node)),
            SyntaxKind::ImportEqualsDeclaration => {
                let module_reference = import_equals_declaration_module_reference(node);
                if ast::is_external_module_reference(&module_reference) {
                    module_reference.expression().cloned()
                } else {
                    None
                }
            }
            SyntaxKind::NamespaceImport => get_module_specifier_from_node(&parent_node(&parent_node(node))),
            SyntaxKind::ImportSpecifier => {
                get_module_specifier_from_node(&parent_node(&parent_node(&parent_node(node))))
            }
            SyntaxKind::NamespaceExport => get_module_specifier_from_node(&parent_node(node)),
            SyntaxKind::ExportSpecifier => get_module_specifier_from_node(&parent_node(&parent_node(node))),
            _ => panic!("Unhandled case in getModuleSpecifierForImportOrExport"),
        }
    }

    pub fn get_mutable_array_or_tuple_type(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.intersects(TypeFlags::Union) {
            return map_type_self(self, t, &mut |c, t| Some(c.get_mutable_array_or_tuple_type(t)))
                .unwrap_or_else(|| Arc::clone(t));
        }
        if t.flags.intersects(TypeFlags::Any)
            || self.is_mutable_array_or_tuple(&self.get_base_constraint_or_type(t))
        {
            return Arc::clone(t);
        }
        if is_tuple_type(t) {
            let element_infos = t
                .target_tuple_type()
                .map(|d| d.element_infos.clone())
                .unwrap_or_default();
            let element_types = self.get_element_types(t);
            return self.create_tuple_type_ex(element_types, element_infos, false);
        }
        self.create_tuple_type_ex(
            vec![Arc::clone(t)],
            vec![TupleElementInfo {
                label: None,
                flags: ElementFlags::Variadic,
                labeled_declaration: None,
                type_: None,
            }],
            false,
        )
    }

    pub fn get_next_base_constraint(&mut self, t: &Arc<Type>, stack: &[RecursionId]) -> Option<Arc<Type>> {
        let constraint = self.get_resolved_base_constraint(t, stack);
        if Arc::ptr_eq(&constraint, &self.no_constraint_type())
            || Arc::ptr_eq(&constraint, &self.circular_constraint_type())
        {
            return None;
        }
        Some(constraint)
    }

    pub fn get_no_infer_type(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if self.is_no_infer_target_type(t) {
            return self.get_or_create_substitution_type(t, &self.unknown_type());
        }
        Arc::clone(t)
    }

    pub fn get_non_nullable_type_if_needed(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if self.is_nullable_type(t) {
            return self.get_non_nullable_type(t);
        }
        Arc::clone(t)
    }

    pub fn get_non_undefined_type(&mut self, t: &Arc<Type>) -> Arc<Type> {
        let mut type_or_constraint = Arc::clone(t);
        if some_type_self(self, t, |c, t| c.is_generic_type_with_undefined_constraint(t)) {
            type_or_constraint = map_type_self(self, t, &mut |c: &mut Checker, t: &Arc<Type>| {
                if t.flags.intersects(TYPE_FLAGS_INSTANTIABLE) {
                    Some(c.get_base_constraint_or_type(t))
                } else {
                    Some(Arc::clone(t))
                }
            })
            .unwrap_or_else(|| Arc::clone(t));
        }
        self.get_type_with_facts(&type_or_constraint, TypeFacts::NE_UNDEFINED)
    }

    pub fn get_normalized_tuple_type(&mut self, t: &Arc<Type>, writing: bool) -> Arc<Type> {
        let elements = self.get_element_types(t);
        let normalized_elements: Vec<Arc<Type>> = elements
            .iter()
            .map(|e| {
                if e.flags.intersects(TYPE_FLAGS_SIMPLIFIABLE) {
                    self.get_simplified_type(e, writing)
                } else {
                    Arc::clone(e)
                }
            })
            .collect();
        if !same_types(&elements, &normalized_elements) {
            let target = t.target().cloned().unwrap_or_else(|| Arc::clone(t));
            return self.create_normalized_tuple_type(&target, &normalized_elements);
        }
        Arc::clone(t)
    }

    pub fn get_normalized_type(&mut self, t: &Arc<Type>, writing: bool) -> Arc<Type> {
        let mut current = Arc::clone(t);
        loop {
            let next = if is_fresh_literal_type(&current) {
                literal_type_regular_type(&current)
            } else if is_generic_tuple_type(&current) {
                self.get_normalized_tuple_type(&current, writing)
            } else if current.object_flags.intersects(OBJECT_FLAGS_REFERENCE) {
                self.get_single_base_for_non_augmenting_subtype(&current)
                    .unwrap_or_else(|| Arc::clone(&current))
            } else if current
                .flags
                .intersects(TypeFlags::Union | TypeFlags::Intersection)
            {
                self.get_normalized_union_or_intersection_type(&current, writing)
            } else if current.flags.intersects(TypeFlags::Substitution) {
                if writing {
                    substitution_base_type(&current).unwrap_or_else(|| Arc::clone(&current))
                } else {
                    self.get_substitution_intersection(&current)
                }
            } else if current.flags.intersects(TYPE_FLAGS_SIMPLIFIABLE) {
                self.get_simplified_type(&current, writing)
            } else {
                return current;
            };
            if Arc::ptr_eq(&next, &current) {
                return next;
            }
            current = next;
        }
    }

    pub fn get_normalized_union_or_intersection_type(&mut self, t: &Arc<Type>, writing: bool) -> Arc<Type> {
        let reduced = self.get_reduced_type(t);
        if !Arc::ptr_eq(&reduced, t) {
            return reduced;
        }
        if t.flags.intersects(TypeFlags::Intersection) && self.should_normalize_intersection(t) {
            let types = t.types().unwrap_or(&[]).to_vec();
            let normalized_types: Vec<Arc<Type>> = types
                .iter()
                .map(|u| self.get_normalized_type(u, writing))
                .collect();
            if !same_types(&normalized_types, &types) {
                return self.get_intersection_type(normalized_types);
            }
        }
        Arc::clone(t)
    }
}

pub fn get_mapped_type_modifiers(t: &Arc<Type>) -> MappedTypeModifiers {
    let declaration = mapped_type_declaration(t);
    let mut modifiers = MappedTypeModifiers::empty();
    if let Some(readonly_token) = mapped_declaration_readonly_token(&declaration) {
        modifiers.insert(if readonly_token.kind == SyntaxKind::MinusToken {
            MappedTypeModifiers::ExcludeReadonly
        } else {
            MappedTypeModifiers::IncludeReadonly
        });
    }
    if let Some(question_token) = mapped_declaration_question_token(&declaration) {
        modifiers.insert(if question_token.kind == SyntaxKind::MinusToken {
            MappedTypeModifiers::ExcludeOptional
        } else {
            MappedTypeModifiers::IncludeOptional
        });
    }
    modifiers
}

pub fn get_mapped_type_optionality(t: &Arc<Type>) -> i32 {
    let modifiers = get_mapped_type_modifiers(t);
    if modifiers.intersects(MappedTypeModifiers::ExcludeOptional) {
        return -1;
    }
    if modifiers.intersects(MappedTypeModifiers::IncludeOptional) {
        return 1;
    }
    0
}

pub fn get_modified_readonly_state(state: bool, modifiers: MappedTypeModifiers) -> bool {
    if modifiers.intersects(MappedTypeModifiers::IncludeReadonly) {
        return true;
    }
    if modifiers.intersects(MappedTypeModifiers::ExcludeReadonly) {
        return false;
    }
    state
}

pub fn get_module_specifier_from_node(node: &Arc<Node>) -> Option<Arc<Node>> {
    match node.kind {
        SyntaxKind::ImportDeclaration | SyntaxKind::JSImportDeclaration => node_module_specifier(node).cloned(),
        SyntaxKind::ExportDeclaration => node_module_specifier(node).cloned(),
        _ => panic!("Unhandled case in getModuleSpecifierFromNode"),
    }
}

pub fn get_name_from_index_info(info: &IndexInfo) -> String {
    if let Some(declaration) = &info.declaration {
        let parameters = node_parameters(declaration)
            .map(|l| l.nodes.as_slice())
            .unwrap_or(&[]);
        if let Some(first) = parameters.first() {
            let name = first.name().unwrap_or(first);
            return scanner_declaration_name_to_string(name);
        }
    }
    "x".to_string()
}

pub fn get_number_literal_value(t: &Arc<Type>) -> f64 {
    literal_type_number_value(t)
}

pub fn get_node_list_key(nodes: &[Arc<Node>]) -> CacheHashKey {
    let mut b = R23KeyBuilder::new();
    b.write_int(nodes.len() as i32);
    for n in nodes {
        b.write_node(Some(n));
    }
    b.hash()
}

pub fn get_type_instantiation_key(
    type_arguments: &[Arc<Type>],
    alias: Option<&TypeAlias>,
    single_signature: bool,
) -> CacheHashKey {
    let mut b = R23KeyBuilder::new();
    b.write_types(type_arguments);
    b.write_alias(alias);
    if single_signature {
        b.write_byte(b'!');
    }
    b.hash()
}
