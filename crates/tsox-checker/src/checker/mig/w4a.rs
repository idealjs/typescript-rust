use crate::checker::checker::*;
use std::sync::Arc;
use crate::checker::mig::m3a_2::{has_dot_dot_dot_token, new_diagnostic_for_node};
use crate::checker::mig::m2e::r19k3_defs::{
    get_candidate_variable_declaration_initializer, resolving_explicit_type_of_symbol_add_if_absent,
    resolving_explicit_type_of_symbol_remove, NodeAccessExtR19k3,
};
use crate::checker::mig::m2h::r22k10_defs::{
    is_access_expression, set_flow_node_of, R22K10FactoryExt,
};
use crate::checker::types::{CheckMode, TypeFlags};
use crate::checker::utilities_get_assignment_target::get_binding_element_property_name;
use crate::checker::utilities_is_optional_symbol::is_fresh_literal_type;
use crate::checker::utilities_token_is_identifier_or_keyword::is_nullable_type;
use super::m2d_3::FlowState;
use super::m2e::r19k3_defs::non_dotted_name_cache_key;
use super::wc1b::every_type;
use super::wc3::NodeAccessExt;
use tsox_core::diagnostics::messages_generated as diagnostics;
use tsox_frontend::ast::mig::m3g_2::is_object_literal_method;
use tsox_frontend::ast::{
    is_array_literal_expression, is_binding_element, is_binding_pattern, is_for_in_statement,
    is_for_of_statement, is_function_expression_or_arrow_function, is_identifier,
    is_jsx_attributes, is_object_binding_pattern, is_object_literal_expression,
    is_parameter_declaration, is_property_assignment, is_variable_declaration, Diagnostic,
    FlowNode, Node, Symbol, SymbolFlags, SyntaxKind, INTERNAL_SYMBOL_NAME_PREFIX,
};

impl Checker {
    pub fn get_discriminant_property_access(
        &mut self,
        f: &FlowState,
        expr: &Arc<Node>,
        computed_type: &Arc<Type>,
    ) -> Option<Arc<Node>> {
        let declared_type = f.declared_type.as_ref().unwrap();
        if declared_type.flags.intersects(TypeFlags::Union)
            || computed_type.flags.intersects(TypeFlags::Union)
        {
            let access = self.get_candidate_discriminant_property_access(f, expr);
            if let Some(access) = access {
                if let Some(name) = self.get_accessed_property_name(&access) {
                    let mut t = Arc::clone(computed_type);
                    if declared_type.flags.intersects(TypeFlags::Union)
                        && self.is_type_subset_of(computed_type, declared_type)
                    {
                        t = Arc::clone(declared_type);
                    }
                    if self.is_discriminant_property(&t, &name) {
                        return Some(access);
                    }
                }
            }
        }
        None
    }

    pub fn get_candidate_discriminant_property_access(
        &mut self,
        f: &FlowState,
        expr: &Arc<Node>,
    ) -> Option<Arc<Node>> {
        if f.reference.as_deref().is_some_and(is_binding_pattern)
            || f.reference
                .as_deref()
                .is_some_and(is_function_expression_or_arrow_function)
            || f.reference.as_deref().is_some_and(is_object_literal_method)
        {
            if is_identifier(expr) {
                let Some(symbol) = self.get_resolved_symbol(expr) else {
                    return None;
                };
                let declaration = self
                    .get_export_symbol_of_value_symbol_if_exported(&symbol)
                    .value_declaration
                    .clone();
                if let Some(declaration) = declaration {
                    if (is_binding_element(&declaration) || is_parameter_declaration(&declaration))
                        && declaration.parent().as_ref().is_some_and(|p| {
                            f.reference.as_ref().is_some_and(|r| Arc::ptr_eq(r, p))
                        })
                        && declaration.initializer().is_none()
                        && !has_dot_dot_dot_token(&declaration)
                    {
                        return Some(declaration);
                    }
                }
            }
        } else if is_access_expression(expr) {
            if self.is_matching_reference(f.reference.as_ref().unwrap(), expr.expression().unwrap()) {
                return Some(Arc::clone(expr));
            }
        } else if is_identifier(expr) {
            let Some(symbol) = self.get_resolved_symbol(expr) else {
                return None;
            };
            if self.is_constant_variable(&symbol) {
                let declaration = symbol.value_declaration.clone();
                let mut initializer = get_candidate_variable_declaration_initializer(
                    declaration.as_ref(),
                );
                if let Some(initializer) = initializer.clone() {
                    if is_access_expression(&initializer)
                        && self.is_matching_reference(f.reference.as_ref().unwrap(), initializer.expression().unwrap())
                    {
                        return Some(initializer);
                    }
                }
                if let Some(declaration) = declaration {
                    if is_binding_element(&declaration) && declaration.initializer().is_none() {
                        initializer = get_candidate_variable_declaration_initializer(Some(
                            &declaration.parent().unwrap().parent().unwrap(),
                        ));
                        if let Some(init) = initializer {
                            if (is_identifier(&init) || is_access_expression(&init))
                                && self.is_matching_reference(f.reference.as_ref().unwrap(), &init)
                            {
                                return Some(declaration);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    pub fn get_assigned_type(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let parent = node.parent().unwrap();
        match parent.kind {
            SyntaxKind::ForInStatement => self.string_type(),
            SyntaxKind::ForOfStatement => {
                if let Some(t) = self.check_right_hand_side_of_for_of(&parent) {
                    return t;
                }
                self.error_type()
            }
            SyntaxKind::BinaryExpression => self.get_assigned_type_of_binary_expression(&parent),
            SyntaxKind::DeleteExpression => self.undefined_type(),
            SyntaxKind::ArrayLiteralExpression => {
                self.get_assigned_type_of_array_literal_element(&parent, node)
            }
            SyntaxKind::SpreadElement => self.get_assigned_type_of_spread_expression(&parent),
            SyntaxKind::PropertyAssignment => self.get_assigned_type_of_property_assignment(&parent),
            SyntaxKind::ShorthandPropertyAssignment => {
                self.get_assigned_type_of_shorthand_property_assignment(&parent)
            }
            _ => self.error_type(),
        }
    }

    pub fn get_assigned_type_of_binary_expression(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let is_destructuring_default_assignment =
            (is_array_literal_expression(&node.parent().unwrap())
                && self.is_destructuring_assignment_target(&node.parent().unwrap()))
                || (is_property_assignment(&node.parent().unwrap())
                    && self
                        .is_destructuring_assignment_target(&node.parent().unwrap().parent().unwrap()));
        if is_destructuring_default_assignment {
            let binary = node.as_binary_expression_node();
            let assigned_type = self.get_assigned_type(node);
            return self.get_type_with_default(&assigned_type, Some(&binary.right));
        }
        let binary = node.as_binary_expression_node();
        self.check_expression_ex(&binary.right, CheckMode::Normal)
    }

    pub fn get_assigned_type_of_array_literal_element(
        &mut self,
        node: &Arc<Node>,
        element: &Arc<Node>,
    ) -> Arc<Type> {
        let assigned = self.get_assigned_type(node);
        let index = node
            .as_array_literal_expression_node()
            .elements
            .nodes
            .iter()
            .position(|e| Arc::ptr_eq(e, element))
            .unwrap_or(usize::MAX);
        self.get_type_of_destructured_array_element(&assigned, index)
    }

    pub fn get_assigned_type_of_property_assignment(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let assigned = self.get_assigned_type(&node.parent().unwrap());
        self.get_type_of_destructured_property(&assigned, node.name().unwrap())
    }

    pub fn get_assigned_type_of_shorthand_property_assignment(
        &mut self,
        node: &Arc<Node>,
    ) -> Arc<Type> {
        let assigned = self.get_assigned_type_of_property_assignment(node);
        let initializer = node
            .as_shorthand_property_assignment_node()
            .object_assignment_initializer
            .clone();
        self.get_type_with_default_opt(&assigned, initializer.as_ref())
    }

    pub fn get_assignment_reduced_type_worker(
        &mut self,
        declared_type: &Arc<Type>,
        assigned_type: &Arc<Type>,
    ) -> Arc<Type> {
        let checker_ptr: *mut Checker = self;
        let filtered_type = unsafe {
            (*checker_ptr).filter_type(declared_type, &mut |t: &Arc<Type>| {
                (*checker_ptr).type_maybe_assignable_to(assigned_type, t)
            })
        };
        let mut reduced_type = Arc::clone(&filtered_type);
        if assigned_type.flags.intersects(TypeFlags::BooleanLiteral)
            && is_fresh_literal_type(assigned_type)
        {
            reduced_type = unsafe {
                (*checker_ptr).map_type(&filtered_type, &mut |t: &Arc<Type>| {
                    Some((*checker_ptr).get_fresh_type_of_literal_type(t))
                })
            }
            .unwrap_or_else(|| Arc::clone(&filtered_type));
        }
        if self.is_type_assignable_to(assigned_type, &reduced_type) {
            reduced_type
        } else {
            Arc::clone(declared_type)
        }
    }

    pub fn get_element_type_of_evolving_array_type(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if t.object_flags.intersects(ObjectFlags::EvolvingArray) {
            if let Some(evolving) = t.as_evolving_array_type() {
                if let Some(element_type) = &evolving.element_type {
                    return Arc::clone(element_type);
                }
            }
        }
        self.never_type()
    }

    pub fn get_final_array_type(&mut self, t: &Arc<Type>) -> Arc<Type> {
        let Some(evolving) = t.as_evolving_array_type() else {
            return self.never_type();
        };
        if let Some(final_array_type) = evolving.final_array_type.get().cloned() {
            return final_array_type;
        }
        self.create_final_array_type(evolving.element_type.as_ref().unwrap())
    }

    pub fn get_explicit_type_of_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
        diagnostic: Option<&mut Diagnostic>,
    ) -> Option<Arc<Type>> {
        let symbol = self.resolve_symbol(symbol);
        if !resolving_explicit_type_of_symbol_add_if_absent(&symbol) {
            return None;
        }
        let result = self.get_explicit_type_of_symbol_worker(&symbol, diagnostic);
        resolving_explicit_type_of_symbol_remove(&symbol);
        result
    }

    fn get_explicit_type_of_symbol_worker(
        &mut self,
        symbol: &Arc<Symbol>,
        diagnostic: Option<&mut Diagnostic>,
    ) -> Option<Arc<Type>> {
        if symbol.flags.intersects(
            SymbolFlags::Function
                | SymbolFlags::Method
                | SymbolFlags::Class
                | SymbolFlags::ValueModule,
        ) {
            return Some(self.get_type_of_symbol(symbol));
        }
        if symbol
            .flags
            .intersects(SymbolFlags::VARIABLE | SymbolFlags::Property)
        {
            if symbol
                .check_flags
                .intersects(CheckFlags::Mapped)
            {
                let origin = self
                    .mapped_symbol_links
                    .get(symbol)
                    .and_then(|l| l.synthetic_origin.clone());
                if let Some(origin) = origin {
                    if self.get_explicit_type_of_symbol(&origin, None).is_some() {
                        return Some(self.get_type_of_symbol(symbol));
                    }
                }
            }
            let declaration = symbol.value_declaration.clone();
            if let Some(declaration) = declaration {
                if self.is_declaration_with_explicit_type_annotation(&declaration) {
                    return Some(self.get_type_of_symbol(symbol));
                }
                if is_variable_declaration(&declaration)
                    && is_for_of_statement(&declaration.parent().unwrap().parent().unwrap())
                {
                    let statement = declaration.parent().unwrap().parent().unwrap();
                    let expression_type =
                        self.get_type_of_dotted_name(statement.expression().unwrap(), None);
                    if let Some(expression_type) = expression_type {
                        let use_ = if statement
                            .as_for_in_or_of_statement()
                            .await_modifier
                            .is_some()
                        {
                            IterationUse::ForOf { for_await: true }
                        } else {
                            IterationUse::ForOf { for_await: false }
                        };
                        return Some(self.check_iterated_type_or_element_type(
                            use_,
                            &expression_type,
                            None,
                        ));
                    }
                }
                if let Some(diagnostic) = diagnostic {
                    diagnostic.add_related_info(new_diagnostic_for_node(
                        Some(&declaration),
                        diagnostics::X_0_NEEDS_AN_EXPLICIT_TYPE_ANNOTATION,
                        vec![self.symbol_to_string(symbol)],
                    ));
                }
            }
        }
        None
    }

    pub fn get_initial_type(&mut self, node: &Arc<Node>) -> Arc<Type> {
        match node.kind {
            SyntaxKind::VariableDeclaration => self.get_initial_type_of_variable_declaration(node),
            SyntaxKind::BindingElement => self.get_initial_type_of_binding_element(node),
            _ => panic!("Unhandled case in get_initial_type"),
        }
    }

    pub fn get_initial_type_of_variable_declaration(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(initializer) = node.initializer() {
            return self.get_type_of_initializer(&initializer);
        }
        let grand = node.parent().unwrap().parent().unwrap();
        if is_for_in_statement(&grand) {
            return self.string_type();
        }
        if is_for_of_statement(&grand) {
            if let Some(t) = self.check_right_hand_side_of_for_of(&grand) {
                return t;
            }
        }
        self.error_type()
    }

    pub fn get_initial_type_of_binding_element(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let pattern = node.parent().unwrap();
        let parent_type = self.get_initial_type(&pattern.parent().unwrap());
        let t = if is_object_binding_pattern(&pattern) {
            self.get_type_of_destructured_property(
                &parent_type,
                &get_binding_element_property_name(node).unwrap(),
            )
        } else if !has_dot_dot_dot_token(node) {
            let index = pattern
                .as_binding_pattern_node()
                .elements
                .nodes
                .iter()
                .position(|e| Arc::ptr_eq(e, node))
                .unwrap_or(usize::MAX);
            self.get_type_of_destructured_array_element(&parent_type, index)
        } else {
            self.get_type_of_destructured_spread_expression(&parent_type)
        };
        self.get_type_with_default_opt(&t, node.initializer())
    }

    pub fn get_flow_reference_key(&mut self, f: &FlowState) -> CacheHashKey {
        let mut b = KeyBuilder::new();
        if self.write_flow_cache_key(
            &mut b,
            f.reference.as_ref().unwrap(),
            f.declared_type.as_ref().unwrap(),
            f.initial_type.as_ref(),
            f.flow_container.as_ref(),
        ) {
            return b.hash();
        }
        non_dotted_name_cache_key()
    }

    pub fn get_flow_type_in_constructor(
        &mut self,
        symbol: &Arc<Symbol>,
        constructor: &Arc<Node>,
    ) -> Option<Arc<Type>> {
        let access_name = if symbol.name.starts_with(
            format!("{}#", INTERNAL_SYMBOL_NAME_PREFIX).as_str(),
        ) {
            self.factory
                .new_private_identifier(&symbol.name[symbol.name.find('@').unwrap() + 1..])
        } else {
            self.factory.new_identifier(&symbol.name)
        };
        let this_keyword = self.factory.new_keyword_expression(SyntaxKind::ThisKeyword);
        let reference = self.factory.new_property_access_expression(
            &this_keyword,
            None,
            &access_name,
            NodeFlags::empty(),
        );
        reference
            .expression()
            .unwrap()
            .set_parent(&reference);
        reference.set_parent(constructor);
        set_flow_node_of(
            &reference,
            self.program
                .symbol_map()
                .end_flow_node_of(constructor)
                .cloned(),
        );
        let flow_type = self.get_flow_type_of_property(&reference, Some(symbol));
        if self.no_implicit_any
            && (Arc::ptr_eq(&flow_type, &self.auto_type())
                || Arc::ptr_eq(&flow_type, &self.auto_array_type()))
        {
            if let Some(value_declaration) = symbol.value_declaration.as_ref() {
                let source_file = self.get_source_file_of_node(value_declaration);
                let symbol_text = self.symbol_to_string(symbol);
                let flow_type_text = self.type_to_string(&flow_type);
                self.diagnostics.add(Diagnostic::new(
                    source_file,
                    value_declaration.loc,
                    diagnostics::MEMBER_0_IMPLICITLY_HAS_AN_1_TYPE,
                    vec![symbol_text, flow_type_text],
                ));
            }
        }
        if every_type(&flow_type, &mut |t: &Arc<Type>| is_nullable_type(t)) {
            return None;
        }
        Some(self.convert_auto_to_any(&flow_type))
    }

    pub fn get_flow_type_in_static_blocks(
        &mut self,
        symbol: &Arc<Symbol>,
        static_blocks: &[Arc<Node>],
    ) -> Option<Arc<Type>> {
        let access_name = if symbol.name.starts_with(
            format!("{}#", INTERNAL_SYMBOL_NAME_PREFIX).as_str(),
        ) {
            self.factory
                .new_private_identifier(&symbol.name[symbol.name.find('@').unwrap() + 1..])
        } else {
            self.factory.new_identifier(&symbol.name)
        };
        for static_block in static_blocks {
            let this_keyword = self.factory.new_keyword_expression(SyntaxKind::ThisKeyword);
            let reference = self.factory.new_property_access_expression(
                &this_keyword,
                None,
                &access_name,
                NodeFlags::empty(),
            );
            reference
                .expression()
                .unwrap()
                .set_parent(&reference);
            reference.set_parent(static_block);
            set_flow_node_of(
                &reference,
                self.program
                    .symbol_map()
                    .flow_node_of(static_block)
                    .cloned(),
            );
            let flow_type = self.get_flow_type_of_property(&reference, Some(symbol));
            if self.no_implicit_any
                && (Arc::ptr_eq(&flow_type, &self.auto_type())
                    || Arc::ptr_eq(&flow_type, &self.auto_array_type()))
            {
                if let Some(value_declaration) = symbol.value_declaration.as_ref() {
                    let source_file = self.get_source_file_of_node(value_declaration);
                    let symbol_name = self.symbol_to_string(symbol);
                    let flow_type_name = self.type_to_string(&flow_type);
                    self.diagnostics.add(Diagnostic::new(
                        source_file,
                        value_declaration.loc,
                        diagnostics::MEMBER_0_IMPLICITLY_HAS_AN_1_TYPE,
                        vec![symbol_name, flow_type_name],
                    ));
                }
            }
            if every_type(&flow_type, &mut |t: &Arc<Type>| is_nullable_type(t)) {
                continue;
            }
            return Some(self.convert_auto_to_any(&flow_type));
        }
        None
    }

    pub fn get_apparent_type_of_contextual_type(
        &mut self,
        node: &Arc<Node>,
        context_flags: ContextFlags,
    ) -> Option<Arc<Type>> {
        let contextual_type = if is_object_literal_method(node) {
            self.get_contextual_type_for_object_literal_method(node, context_flags)
        } else {
            self.get_contextual_type(node, context_flags)
        };
        let Some(contextual_type) = contextual_type else {
            return None;
        };
        let instantiated_type =
            self.instantiate_contextual_type(&contextual_type, node, context_flags);
        if !(context_flags.intersects(ContextFlags::NoConstraints)
            && instantiated_type.flags.intersects(TypeFlags::TYPE_VARIABLE))
        {
            let checker_ptr: *mut Checker = self;
            let apparent_type = unsafe {
                (*checker_ptr).map_type_ex(
                    &instantiated_type,
                    &mut |t: &Arc<Type>| {
                        if t.object_flags.intersects(ObjectFlags::Mapped) {
                            Some(Arc::clone(t))
                        } else {
                            Some(unsafe { (*checker_ptr).get_apparent_type(t) })
                        }
                    },
                    true,
                )
            };
            let apparent_type = apparent_type?;
            if apparent_type.flags.intersects(TypeFlags::Union)
                && is_object_literal_expression(node)
            {
                return Some(
                    self.discriminate_contextual_type_by_object_members(node, &apparent_type),
                );
            }
            if apparent_type.flags.intersects(TypeFlags::Union) && is_jsx_attributes(node) {
                return self.discriminate_contextual_type_by_jsx_attributes(node, &apparent_type);
            }
            return Some(apparent_type);
        }
        None
    }
}
