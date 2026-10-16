#![allow(unused_imports)]
#![allow(dead_code)]

#[path = "r21k7_defs.rs"]
pub(crate) mod r21k7_defs;
pub(crate) use r21k7_defs::*;

use crate::checker::checker::*;
use crate::checker::types::*;
use std::sync::Arc;
use tsox_frontend::ast::{self as tst, Node, NodeData, Symbol, SymbolFlags, SyntaxKind};
use tsox_frontend::ast::{
    get_containing_class, is_in_js_file, is_jsx_attributes, is_jsx_self_closing_element, is_class_declaration, is_interface_declaration, is_qualified_name, is_function_like, skip_parentheses,
};
use tsox_frontend::ast::mig::m3g_3::{skip_outer_expressions, OuterExpressionKinds};
use crate::checker::checker_this_container::get_this_container;
use crate::checker::mig::m1f_4::{get_target_type, get_this_parameter_from_node_context};

impl Checker {
    pub(crate) fn get_deprecated_suggestion_node(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_deprecated_suggestion_node"); 
        let node = skip_parentheses(node);
        match node.kind {
            SyntaxKind::CallExpression | SyntaxKind::Decorator | SyntaxKind::NewExpression => {
                node.expression()
                    .and_then(|e| self.get_deprecated_suggestion_node(e))
            }
            SyntaxKind::TaggedTemplateExpression => {
                if let NodeData::TaggedTemplateExpression(data) = &node.data {
                    self.get_deprecated_suggestion_node(&data.tag)
                } else {
                    None
                }
            }
            SyntaxKind::JsxOpeningElement | SyntaxKind::JsxSelfClosingElement => {
                self.get_deprecated_suggestion_node(tsox_frontend::ast::mig::m3c::tag_name(&node))
            }
            SyntaxKind::ElementAccessExpression => {
                if let NodeData::ElementAccessExpression(data) = &node.data {
                    Some(data.argument_expression.clone())
                } else {
                    None
                }
            }
            SyntaxKind::PropertyAccessExpression => node.name().cloned(),
            SyntaxKind::TypeReference => {
                if let NodeData::TypeReferenceNode(data) = &node.data {
                    if is_qualified_name(&data.type_name) {
                        if let NodeData::QualifiedName(qualified) = &data.type_name.data {
                            Some(Arc::clone(&qualified.right))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub(crate) fn get_class_or_interface_declarations_of_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("get_class_or_interface_declarations_of_symbol"); 
        symbol
            .declarations
            .iter()
            .filter(|d| is_class_declaration(d) || is_interface_declaration(d))
            .cloned()
            .collect()
    }

    pub(crate) fn get_context_node(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("get_context_node"); 
        if is_jsx_attributes(node) {
            if let Some(parent) = node.parent() {
                if !is_jsx_self_closing_element(&parent) {
                    if let Some(grandparent) = parent.parent() {
                        return grandparent;
                    }
                }
            }
        }
        Arc::clone(node)
    }

    pub(crate) fn get_context_free_type_of_expression(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_context_free_type_of_expression"); 
        let any_type = self.get_any_type();
        self.push_contextual_type(node, &any_type, false);
        let t = self.check_expression_ex(node, CheckMode::SkipContextSensitive);
        self.pop_contextual_type();
        t
    }

    pub(crate) fn get_diagnostic_head_message_for_decorator_resolution(
        &mut self,
        node: &Arc<Node>,
    ) -> &'static tsox_core::diagnostics::Message { ::tsox_core::fntrace::enter("get_diagnostic_head_message_for_decorator_resolution"); 
        let parent_kind = node.parent().map(|p| p.kind).unwrap_or(SyntaxKind::Unknown);
        match parent_kind {
            SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => {
                &tsox_core::diagnostics::messages_generated::UNABLE_TO_RESOLVE_SIGNATURE_OF_CLASS_DECORATOR_WHEN_CALLED_AS_AN_EXPRESSION
            }
            SyntaxKind::Parameter => {
                &tsox_core::diagnostics::messages_generated::UNABLE_TO_RESOLVE_SIGNATURE_OF_PARAMETER_DECORATOR_WHEN_CALLED_AS_AN_EXPRESSION
            }
            SyntaxKind::PropertyDeclaration => {
                &tsox_core::diagnostics::messages_generated::UNABLE_TO_RESOLVE_SIGNATURE_OF_PROPERTY_DECORATOR_WHEN_CALLED_AS_AN_EXPRESSION
            }
            SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor => {
                &tsox_core::diagnostics::messages_generated::UNABLE_TO_RESOLVE_SIGNATURE_OF_METHOD_DECORATOR_WHEN_CALLED_AS_AN_EXPRESSION
            }
            _ => unreachable!("Unhandled case in getDiagnosticHeadMessageForDecoratorResolution"),
        }
    }

    pub(crate) fn get_decorator_argument_count(
        &mut self,
        node: &Arc<Node>,
        signature: &Arc<Signature>,
    ) -> usize { ::tsox_core::fntrace::enter("get_decorator_argument_count"); 
        if self.compiler_options.experimental_decorators.is_true() {
            return self.get_legacy_decorator_argument_count(node, signature) as usize;
        }
        self.get_parameter_count(signature).clamp(1, 2)
    }

    pub(crate) fn get_effective_check_node(&mut self, argument: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("get_effective_check_node"); 
        let flags = if is_in_js_file(argument) {
            OuterExpressionKinds::PARENS
                | OuterExpressionKinds::SATISFIES
                | OuterExpressionKinds::EXCLUDE_JSDOC_TYPE_ASSERTION
        } else {
            OuterExpressionKinds::PARENS | OuterExpressionKinds::SATISFIES
        };
        skip_outer_expressions(argument, flags)
    }

    pub(crate) fn for_each_property(
        &mut self,
        prop: &Arc<Symbol>,
        callback: &mut dyn FnMut(&mut Checker, &Arc<Symbol>) -> bool,
    ) -> bool { ::tsox_core::fntrace::enter("for_each_property"); 
        if !prop.check_flags.contains(CheckFlags::SYNTHETIC) {
            return callback(self, prop);
        }
        let containing_type = self
            .value_symbol_links
            .get(prop)
            .and_then(|l| l.containing_type.clone());
        if let Some(containing_type) = containing_type {
            if let Some(types) = containing_type.types() {
                for t in types {
                    if let Some(p) = self.get_property_of_type(t, &prop.name) {
                        if self.for_each_property(&p, callback) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub(crate) fn get_declaring_class(&mut self, prop: &Arc<Symbol>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_declaring_class"); 
        if let Some(parent) = prop.parent() {
            if parent.flags.intersects(SymbolFlags::Class) {
                let parent_of_symbol = self.get_parent_of_symbol(prop)?;
                return Some(self.get_declared_type_of_symbol(&parent_of_symbol));
            }
        }
        None
    }

    pub(crate) fn for_each_enclosing_class(
        &mut self,
        node: &Arc<Node>,
        callback: &mut dyn FnMut(&Arc<Node>) -> bool,
    ) -> bool { ::tsox_core::fntrace::enter("for_each_enclosing_class"); 
        let mut containing_class = get_containing_class(node);
        while let Some(class_like) = containing_class {
            if callback(&class_like) {
                return true;
            }
            containing_class = get_containing_class(&class_like);
        }
        false
    }

    pub(crate) fn get_enclosing_class_from_this_parameter(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_enclosing_class_from_this_parameter"); 
        let this_parameter = get_this_parameter_from_node_context(node);
        let mut this_type: Option<Arc<Type>> = None;
        if let Some(parameter) = &this_parameter {
            if let Some(type_node) = parameter.type_node() {
                this_type = Some(self.get_type_from_type_node(&type_node));
            }
        }
        if let Some(t) = &this_type {
            if t.flags.contains(TypeFlags::TYPE_PARAMETER) {
                this_type = self.get_constraint_of_type_parameter(t);
            }
        } else {
            let this_container = get_this_container(node, false, false);
            if is_function_like(&this_container) {
                this_type = self.get_contextual_this_parameter_type(&this_container);
            }
        }
        if let Some(t) = this_type {
            if t.object_flags
                .intersects(ObjectFlags::CLASS_OR_INTERFACE | ObjectFlags::Reference)
            {
                return Some(get_target_type(&t));
            }
        }
        None
    }

    pub(crate) fn get_base_types_if_unrelated(
        &mut self,
        left_type: &Arc<Type>,
        right_type: &Arc<Type>,
        is_related: &mut dyn FnMut(&Arc<Type>, &Arc<Type>) -> bool,
    ) -> (Arc<Type>, Arc<Type>) { ::tsox_core::fntrace::enter("get_base_types_if_unrelated"); 
        let mut effective_left = Arc::clone(left_type);
        let mut effective_right = Arc::clone(right_type);
        let left_base = self.get_base_type_of_literal_type(left_type);
        let right_base = self.get_base_type_of_literal_type(right_type);
        if !is_related(&left_base, &right_base) {
            effective_left = left_base;
            effective_right = right_base;
        }
        (effective_left, effective_right)
    }

    pub(crate) fn get_adjusted_type_with_facts(
        &mut self,
        t: &Arc<Type>,
        facts: TypeFacts,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_adjusted_type_with_facts"); 
        if facts == TypeFacts::NONE {
            return self.never_type();
        }
        let t = if t.flags.contains(TypeFlags::Conditional) {
            self.distribute_conditional_type_over_facts(t, facts)
        } else {
            Arc::clone(t)
        };
        let adjusted = self.adjust_type_with_facts(&t, facts);
        self.get_regular_type_of_object_type(&adjusted)
    }
}
