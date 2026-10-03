#![allow(unused_imports)]
#[path = "r19k9_defs.rs"]
pub mod r19k9_defs;
pub use r19k9_defs::R19K9NodeExt;
#[path = "r24k9_defs.rs"]
pub mod r24k9_defs;
#[path = "r25k6_defs.rs"]
pub mod r25k6_defs;
use self::r24k9_defs::is_resolving_default_sentinel;
use super::r18k6_defs::InternalSymbolName;
use crate::checker::mig::m1c_6::r21k5_defs::get_recursion_identity;
use crate::checker::relater_recursion_identity::RecursionIdentity as RecursionId;
use crate::checker::types_type_flags_instantiable_non_primitive::OBJECT_FLAGS_PROPAGATING_FLAGS;

#[allow(unused_imports, ambiguous_glob_reexports)]
use crate::checker::*;
#[allow(unused_imports)]
use tsox_frontend::ast::*;
#[allow(unused_imports)]
use tsox_core::diagnostics::messages_generated::*;

#[allow(unused_imports)]
use crate::checker::mig::m1e::r26k3_defs::every_type_with_checker;
pub(crate) use crate::checker::checker::*;
use std::sync::Arc;

pub(crate) struct WideningContext {
    pub parent: Option<Box<WideningContext>>,
    pub property_name: String,
    pub siblings: Option<Vec<Arc<Type>>>,
    pub resolved_properties: Option<Vec<Arc<Symbol>>>,
    pub child_contexts: HashMap<String, Box<WideningContext>>,
    pub widened_types: HashMap<TypeId, Arc<Type>>,
}

impl Checker {
    pub fn get_parent_type_of_class_element(&mut self, node: &Node) -> Arc<Type> { ::tsox_core::fntrace::enter("get_parent_type_of_class_element"); 
        let class_symbol = node
            .parent()
            .and_then(|p| self.get_symbol_of_node(&p))
            .unwrap_or_else(|| self.unknown_symbol());
        if is_static(node) {
            return self.get_type_of_symbol(&class_symbol);
        }
        self.get_declared_type_of_symbol(&class_symbol)
    }

    pub fn get_private_identifier_property_of_type(
        &mut self,
        left_type: &Arc<Type>,
        lexically_scoped_identifier: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_private_identifier_property_of_type"); 
        self.get_property_of_type(left_type, &lexically_scoped_identifier.name)
    }

    pub fn get_properties_of_context(&mut self, context: &mut WideningContext) -> Vec<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_properties_of_context"); 
        if context.resolved_properties.is_none() {
            let mut names: HashMap<String, Arc<Symbol>> = HashMap::new();
            let siblings = self.get_siblings_of_context(context);
            for t in siblings {
                if is_object_literal_type(&t)
                    && !t.object_flags.intersects(ObjectFlags::ContainsSpread)
                {
                    for prop in self.get_properties_of_type(&t) {
                        names.insert(prop.name.clone(), prop);
                    }
                }
            }
            let props: Vec<Arc<Symbol>> = names.into_values().collect();
            context.resolved_properties = Some(props);
        }
        context.resolved_properties.clone().unwrap_or_default()
    }

    pub fn get_propagating_flags_of_types(
        &self,
        types: &[Arc<Type>],
        exclude_kinds: TypeFlags,
    ) -> ObjectFlags { ::tsox_core::fntrace::enter("get_propagating_flags_of_types"); 
        let mut result = ObjectFlags::None;
        for t in types {
            if !t.flags.intersects(exclude_kinds) {
                result |= t.object_flags;
            }
        }
        result & OBJECT_FLAGS_PROPAGATING_FLAGS
    }

    pub fn get_promised_type_of_promise_ex(
        &mut self,
        t: &Arc<Type>,
        error_node: Option<&Arc<Node>>,
        this_type_for_error_out: Option<&mut Option<Arc<Type>>>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_promised_type_of_promise_ex"); 
        if is_type_any(t) {
            return None;
        }
        let key = CachedTypeKey {
            kind: CachedTypeKind::PromisedTypeOfPromise,
            type_id: t.id,
        };
        if let Some(cached) = self.cached_types.get(&key) {
            return Some(cached.clone());
        }
        if self.is_reference_to_type(t, &self.get_global_promise_type()) {
            let result = self.get_type_arguments(t)[0].clone();
            self.cached_types.insert(key, result.clone());
            return Some(result);
        }
        if self.all_types_assignable_to_kind(
            &self.get_base_constraint_or_type(t),
            TypeFlags::PRIMITIVE | TypeFlags::Never,
        ) {
            return None;
        }
        let Some(then_function) = self.get_type_of_property_of_type(t, "then") else {
            return None;
        };
        if is_type_any(&then_function) {
            return None;
        }
        let then_signatures = self.get_signatures_of_type(&then_function, SignatureKind::Call);
        if then_signatures.is_empty() {
            if let Some(error_node) = error_node {
                let _ = self.error_message(error_node,A_PROMISE_MUST_HAVE_A_THEN_METHOD, &[]);
            }
            return None;
        }
        let mut this_type_for_error: Option<Arc<Type>> = None;
        let mut candidates: Vec<Arc<Signature>> = vec![];
        for then_signature in &then_signatures {
            let this_type = self.get_this_type_of_signature(then_signature);
            match this_type {
                Some(this_type) if this_type.id != self.void_type().id => {
                    if !self.is_type_related_to(t, &this_type, RelationKind::Subtype) {
                        this_type_for_error = Some(this_type);
                    } else {
                        candidates.push(then_signature.clone());
                    }
                }
                _ => candidates.push(then_signature.clone()),
            }
        }
        if candidates.is_empty() {
            if let Some(out) = this_type_for_error_out {
                *out = this_type_for_error.clone();
            }
            if let Some(error_node) = error_node {
                let this_type_text = self.type_to_string(t);
                let target_text = self.type_to_string(this_type_for_error.as_ref().unwrap());
                let _ = self.error_message(
                    error_node,THE_THIS_CONTEXT_OF_TYPE_0_IS_NOT_ASSIGNABLE_TO_METHOD_S_THIS_OF_TYPE_1,
                    &[this_type_text, target_text],
                );
            }
            return None;
        }
        let first_param_types: Vec<Arc<Type>> = candidates
            .iter()
            .map(|sig| self.get_type_of_first_parameter_of_signature(sig))
            .collect();
        let first_param_union = self.get_union_type(first_param_types);
        let onfulfilled_parameter_type =
            self.get_type_with_facts(&first_param_union, TypeFacts::NE_UNDEFINED_OR_NULL);
        if is_type_any(&onfulfilled_parameter_type) {
            return None;
        }
        let onfulfilled_parameter_signatures =
            self.get_signatures_of_type(&onfulfilled_parameter_type, SignatureKind::Call);
        if onfulfilled_parameter_signatures.is_empty() {
            if let Some(error_node) = error_node {
                let _ = self.error_message(
                    error_node,THE_FIRST_PARAMETER_OF_THE_THEN_METHOD_OF_A_PROMISE_MUST_BE_A_CALLBACK,
                    &[],
                );
            }
            return None;
        }
        let result_first_params: Vec<Arc<Type>> = onfulfilled_parameter_signatures
            .iter()
            .map(|sig| self.get_type_of_first_parameter_of_signature(sig))
            .collect();
        let result = self.get_union_type_ex(result_first_params, UnionReduction::Subtype);
        self.cached_types.insert(key, result.clone());
        Some(result)
    }

    pub fn get_properties_of_object_type(&mut self, t: &Arc<Type>) -> Vec<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_properties_of_object_type"); 
        if t.flags.intersects(TypeFlags::Object) {
            let resolved = self.resolve_structured_type_members(t);
            return resolved
                .as_structured()
                .map(|d| d.properties.clone())
                .unwrap_or_default();
        }
        vec![]
    }

    pub fn get_properties_of_union_or_intersection_type(
        &mut self,
        t: &Arc<Type>,
    ) -> Vec<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_properties_of_union_or_intersection_type"); 
        let (types, is_union, is_intersection) = match &t.data {
            TypeData::Union(d) => (d.union_or_intersection.types.clone(), true, false),
            TypeData::Intersection(d) => (d.union_or_intersection.types.clone(), false, true),
            _ => return vec![],
        };
        let mut checked: HashSet<String> = HashSet::new();
        let mut props: Vec<Arc<Symbol>> = vec![];
        for current in &types {
            for prop in self.get_properties_of_type(current) {
                if !checked.contains(&prop.name) {
                    checked.insert(prop.name.clone());
                    let combined_prop = self.get_property_of_union_or_intersection_type(t, &prop.name);
                    if let Some(combined_prop) = combined_prop {
                        props.push(combined_prop);
                    }
                }
            }
            if is_union && self.get_index_infos_of_type(current).is_empty() {
                break;
            }
        }
        props
    }

    pub fn get_property_name_from_binding_element(&mut self, e: &Node) -> String { ::tsox_core::fntrace::enter("get_property_name_from_binding_element"); 
        let Some(expr_type) = self.get_literal_type_from_property_name(&e.property_name_or_name())
        else {
            return InternalSymbolName::Missing.to_string();
        };
        if is_type_usable_as_property_name(&expr_type) {
            return get_property_name_from_type(&expr_type);
        }
        InternalSymbolName::Missing.to_string()
    }

    pub fn get_property_of_object_type(
        &mut self,
        t: &Arc<Type>,
        name: &str,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_property_of_object_type"); 
        if t.flags.intersects(TypeFlags::Object) {
            let resolved = self.resolve_structured_type_members(t);
            let symbol = resolved
                .as_structured()
                .and_then(|d| d.members.get(name))
                .cloned();
            if let Some(symbol) = symbol {
                if self.symbol_is_value(&symbol) {
                    return Some(symbol);
                }
            }
        }
        None
    }

    pub fn get_property_of_variable(
        &mut self,
        symbol: &Arc<Symbol>,
        name: &str,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_property_of_variable"); 
        if symbol.flags.intersects(SymbolFlags::VARIABLE) {
            if let Some(value_declaration) = &symbol.value_declaration {
                if let Some(type_annotation) = value_declaration.type_node() {
                    let t = self.get_type_from_type_node(type_annotation);
                    let prop = self.get_property_of_type(&t, name)?;
                    return Some(self.resolve_symbol(&prop));
                }
            }
        }
        None
    }

    pub fn get_quick_type_of_expression(&mut self, node: &Arc<Node>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_quick_type_of_expression"); 
        let expr = skip_parentheses(node);
        if is_await_expression(&expr) {
            if let Some(child) = expr.expression() {
                if let Some(t) = self.get_quick_type_of_expression(child) {
                    return self.get_awaited_type(&t);
                }
            }
            return None;
        }
        if is_call_expression(&expr)
            && expr.expression().is_some_and(|e| e.kind != SyntaxKind::SuperKeyword)
            && !is_require_call(&expr, true)
            && !self.is_symbol_or_symbol_for_call(&expr)
            && !is_import_call(&expr)
        {
            if is_call_chain(&expr) {
                return self.get_return_type_of_single_non_generic_signature_of_call_chain(&expr);
            }
            let func_type = self.check_non_null_expression(expr.expression().expect("expression"));
            return self
                .get_return_type_of_single_non_generic_signature(&func_type, SignatureKind::Call);
        }
        if is_new_expression(&expr) {
            let func_type = self.check_non_null_expression(expr.expression().expect("expression"));
            return self.get_return_type_of_single_non_generic_signature(
                &func_type,
                SignatureKind::Construct,
            );
        }
        if is_assertion_expression(&expr) {
            if let Some(type_node) = expr.type_node() {
                if !is_const_type_reference(type_node) {
                    return Some(self.get_type_from_type_node(type_node));
                }
            }
            return None;
        }
        if is_literal_expression(node) || is_boolean_literal(node) {
            return Some(self.check_expression_ex(node, CheckMode::Normal));
        }
        None
    }

    pub fn get_reduced_union_type(&mut self, union_type: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_reduced_union_type"); 
        let origin_types: &[Arc<Type>] = union_type.types().unwrap_or(&[]);
        let reduced_types: Vec<Arc<Type>> = origin_types
            .iter()
            .map(|t| self.get_reduced_type(t))
            .collect();
        if reduced_types.len() == origin_types.len()
            && reduced_types
                .iter()
                .zip(origin_types.iter())
                .all(|(a, b)| a.id == b.id)
        {
            return union_type.clone();
        }
        let reduced = self.get_union_type(reduced_types);
        if reduced.flags.intersects(TypeFlags::Union) {
            if let TypeData::Union(d) = &reduced.data {
                let _ = d.resolved_reduced_type.set(reduced.clone());
            }
        }
        reduced
    }

    pub fn get_referenced_value_or_alias_symbol(
        &mut self,
        reference: &Arc<Node>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_referenced_value_or_alias_symbol"); 
        let resolved_symbol = self
            .symbol_node_links
            .get(reference)
            .and_then(|l| l.resolved_symbol.clone());
        if let Some(resolved_symbol) = resolved_symbol {
            if resolved_symbol.id() != self.unknown_symbol().id() {
                return Some(resolved_symbol);
            }
        }
        self.resolve_name(
            &reference.text(),
            reference,
            SymbolFlags::VALUE | SymbolFlags::ExportValue | SymbolFlags::Alias,
            false,
        )
    }

    pub fn get_requires_scope_change_cache(&self, node: &Node) -> Tristate { ::tsox_core::fntrace::enter("get_requires_scope_change_cache"); 
        self.node_links
            .get(node)
            .map(|l| l.declaration_requires_scope_change)
            .unwrap_or(Tristate::False)
    }

    pub fn get_resolved_apparent_type_of_mapped_type(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_resolved_apparent_type_of_mapped_type"); 
        let target = match &t.data {
            TypeData::Mapped(d) => d.object.target.clone().unwrap_or_else(|| t.clone()),
            _ => t.clone(),
        };
        let type_variable = self.get_homomorphic_type_variable(&target);
        let target_name_type_none = match &target.data {
            TypeData::Mapped(d) => d.name_type.is_none(),
            _ => false,
        };
        if let Some(type_variable) = type_variable {
            if target_name_type_none {
                let modifiers_type = self.get_modifiers_type_from_mapped_type(t);
                let base_constraint = if self.is_generic_mapped_type(&modifiers_type) {
                    Some(self.get_apparent_type_of_mapped_type(&modifiers_type))
                } else {
                    self.get_base_constraint_of_type(&modifiers_type)
                };
                let mapper = match &t.data {
                    TypeData::Mapped(d) => d.object.mapper.clone(),
                    _ => None,
                };
                if let Some(base_constraint) = base_constraint {
                    if every_type_with_checker(self, &base_constraint, &mut |c: &mut Checker, ct: &Arc<Type>| {
                        c.is_array_or_tuple_type(ct)
                            || c.is_array_or_tuple_or_intersection(ct)
                    }) {
                        let mapping = Arc::new(prepend_type_mapping(
                            Arc::clone(&type_variable),
                            Arc::clone(&base_constraint),
                            mapper.as_deref(),
                        ));
                        return self.instantiate_type(&target, Some(&mapping));
                    }
                }
            }
        }
        t.clone()
    }

    pub fn get_resolved_base_constraint(
        &mut self,
        t: &Arc<Type>,
        stack: &[RecursionId],
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_resolved_base_constraint"); 
        if t.as_constrained_type().is_none() {
            return t.clone();
        }
        if let Some(resolved) = t.resolved_base_constraint() {
            return resolved;
        }
        let mut constraint: Option<Arc<Type>> = None;
        let identity = get_recursion_identity(t);
        if !self.push_type_resolution(
            Arc::as_ptr(t) as *const tsox_frontend::ast::Symbol,
            crate::checker::TypeResolutionProperty::ResolvedBaseConstraint,
        ) {
            return self.circular_constraint_type();
        }
        if stack.len() < 10 || (stack.len() < 50 && !stack.contains(&identity)) {
            let mut new_stack = stack.to_vec();
            new_stack.push(identity);
            let simplified = self.get_simplified_type(t, false);
            constraint = self.compute_base_constraint(&simplified, new_stack);
        }
        if !self.pop_type_resolution() {
            if t.flags.intersects(TypeFlags::TypeParameter) {
                if let Some(error_node) = self.get_constraint_declaration(t) {
                    let type_string = self.type_to_string(t);
                    let _ = self.error_message(
                        &error_node,TYPE_PARAMETER_0_HAS_A_CIRCULAR_CONSTRAINT,
                        &[type_string],
                    );
                }
            }
            constraint = Some(self.circular_constraint_type());
        }
        let constraint = constraint.unwrap_or_else(|| self.no_constraint_type());
        if let Some(existing) = t.resolved_base_constraint() {
            return existing;
        }
        t.set_resolved_base_constraint(constraint.clone());
        constraint
    }

    pub fn get_resolved_members_or_exports_of_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
        resolution_kind: MembersOrExportsResolutionKind,
    ) -> SymbolTable { ::tsox_core::fntrace::enter("get_resolved_members_or_exports_of_symbol"); 
        let is_static = resolution_kind == MembersOrExportsResolutionKind::ResolvedExports;
        let cached = self
            .members_and_exports_links
            .get(symbol)
            .map(|l| {
                if is_static {
                    l.resolved_exports.clone()
                } else {
                    l.resolved_members.clone()
                }
            })
            .filter(|table| !table.is_empty());
        if let Some(cached) = cached {
            return cached;
        }
        let mut early_symbols: SymbolTable = if !is_static {
            symbol.members.clone()
        } else if symbol.flags.intersects(SymbolFlags::MODULE) {
            self.get_exports_of_module_table(symbol)
        } else {
            symbol.exports.clone()
        };
        // Go getResolvedMembersOrExportsOfSymbol：晚绑定循环里的
        // hasLateBindableName 会触发表达式定型，可能重入本函数；先把
        // early symbols 落缓存截断递归，循环结束后覆盖为合并表
        if let Some(l) = self.members_and_exports_links.get_mut(symbol) {
            if is_static {
                l.resolved_exports = early_symbols.clone();
            } else {
                l.resolved_members = early_symbols.clone();
            }
        }
        let mut late_symbols: Option<SymbolTable> = None;
        for decl in &symbol.declarations {
            for member in get_members_of_declaration(decl) {
                if is_static == has_static_modifier(&member) {
                    if self.has_late_bindable_name(&member) {
                        let late = late_symbols.get_or_insert_with(SymbolTable::new);
                        self.late_bind_member(symbol, &mut early_symbols, late, &member);
                    } else if self.has_late_bindable_index_signature(&member) {
                        let late = late_symbols.get_or_insert_with(SymbolTable::new);
                        self.late_bind_index_signature(symbol, &early_symbols, late, &member);
                    }
                }
            }
        }
        if is_static {
            if let Some(assignment_symbol) = symbol
                .exports
                .get(InternalSymbolName::AssignmentDeclaration)
                .cloned()
            {
                for member in &assignment_symbol.declarations {
                    if self.has_late_bindable_name(member) {
                        let late = late_symbols.get_or_insert_with(SymbolTable::new);
                        self.late_bind_member(symbol, &mut early_symbols, late, member);
                    }
                }
            }
        }
        let combined = match &late_symbols {
            Some(late) => self.combine_symbol_tables(&early_symbols, late),
            None => early_symbols.clone(),
        };
        if let Some(l) = self.members_and_exports_links.get_mut(symbol) {
            if is_static {
                l.resolved_exports = combined.clone();
            } else {
                l.resolved_members = combined.clone();
            }
        }
        early_symbols = combined;
        early_symbols
    }

    pub fn get_resolved_symbol_nil(&self, node: &Node) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_resolved_symbol_nil"); 
        self.symbol_node_links
            .get(node)
            .and_then(|l| l.resolved_symbol.clone())
    }

    pub fn get_resolved_type_parameter_default(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_resolved_type_parameter_default"); 
        let (target, mapper) = match &t.data {
            TypeData::TypeParameter(d) => (d.target.clone(), d.mapper.clone()),
            _ => return t.clone(),
        };
        if t.type_parameter_resolved_default().is_none() {
            if let Some(target) = target {
                let target_default = self.get_resolved_type_parameter_default(&target);
                if target_default.id != self.no_constraint_type().id
                    && !is_resolving_default_sentinel(&target_default, t)
                {
                    t.set_type_parameter_resolved_default(
                        self.instantiate_type(&target_default, mapper.as_ref()),
                    );
                    return t.type_parameter_resolved_default().unwrap();
                }
                t.set_type_parameter_resolved_default(self.no_constraint_type());
            } else {
                t.set_type_parameter_resolved_default(self.resolving_default_type());
                let mut default_type = self.no_constraint_type();
                if let Some(symbol) = &t.symbol {
                    let default_declaration = symbol
                        .declarations
                        .iter()
                        .filter_map(|decl| {
                            if is_type_parameter_declaration(decl) {
                                match &decl.data {
                                    NodeData::TypeParameterDeclaration(d) => d.default_type.clone(),
                                    _ => None,
                                }
                            } else {
                                None
                            }
                        })
                        .next();
                    if let Some(default_declaration) = default_declaration {
                        default_type = self.get_type_from_type_node(&default_declaration);
                    }
                }
                if t.type_parameter_resolved_default().unwrap().id
                    == self.resolving_default_type().id
                {
                    t.set_type_parameter_resolved_default(default_type);
                }
            }
        } else if t.type_parameter_resolved_default().unwrap().id == self.resolving_default_type().id
        {
            t.set_type_parameter_resolved_default(self.circular_constraint_type());
        }
        t.type_parameter_resolved_default().unwrap()
    }

    pub fn get_rest_type_of_tuple_type(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_rest_type_of_tuple_type"); 
        let fixed_length = t.target_tuple_type().map(|d| d.fixed_length as i32).unwrap_or(0);
        self.get_element_type_of_slice_of_tuple_type(t, fixed_length, 0, false, false)
            .unwrap_or_else(|| self.unknown_type())
    }

    pub fn get_restrictive_type_parameter(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_restrictive_type_parameter"); 
        let constraint = match &t.data {
            TypeData::TypeParameter(d) => d.constraint.clone(),
            _ => None,
        };
        if (constraint.is_none() && self.get_constraint_declaration(t).is_none())
            || constraint
                .map(|c| c.id == self.no_constraint_type().id)
                .unwrap_or(false)
        {
            return t.clone();
        }
        let key = CachedTypeKey {
            kind: CachedTypeKind::RestrictiveTypeParameter,
            type_id: t.id,
        };
        if let Some(cached) = self.cached_types.get(&key) {
            return cached.clone();
        }
        let mut result = self.new_type_parameter(t.symbol.clone());
        if let Some(result_mut) = Arc::get_mut(&mut result) {
            if let TypeData::TypeParameter(d) = &mut result_mut.data {
                d.constraint = Some(self.no_constraint_type());
            }
        }
        self.cached_types.insert(key, result.clone());
        result
    }
}

pub(crate) fn get_primitive_type_alias_suggestions(
    symbols: &SymbolTable,
) -> Vec<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_primitive_type_alias_suggestions"); 
    let mut result = vec![];
    for (builtin_name, suggestion) in r24k9_defs::primitive_type_alias_suggestions() {
        if symbols.get(builtin_name).is_some() {
            result.push(suggestion);
        }
    }
    result
}
