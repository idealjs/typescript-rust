#![allow(unused_imports)]
#![allow(dead_code)]

use std::collections::HashSet;
use std::sync::Arc;
use tsox_frontend::ast::Node;
use tsox_frontend::ast::SyntaxKind;
use tsox_frontend::ast::node_data_generated::{
    is_identifier, is_omitted_expression, is_postfix_unary_expression, is_prefix_unary_expression,
    is_variable_declaration_list,
};
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::{
    is_binding_pattern, is_function_like_declaration, is_identifier_name,
};
use tsox_frontend::ast::mig::m3g::is_label_name;

use crate::mig::m4i_12::r36k17_defs::NodeAsR36k17Ext;
use crate::mig::m4m::r36k5_defs::NodeDataExt;
use crate::mig::m4e::r38k1_defs::R38K1NodeVisitorExt;
use crate::mig::m4e::r37k1_defs::R37K1DataExt;
use crate::mig::m4e::r39k01_defs::R39K01DataExt;

#[path = "r36k33_defs.rs"]
pub mod r36k33_defs;
pub use r36k33_defs::R36K33NodeAsExt;

pub use crate::printer::{EmitContext, NodeFactory};
pub use tsox_frontend::format::mig::m4o::EmitFlags;
pub use tsox_frontend::ast::visitor::NodeVisitor as Visitor;
pub use tsox_frontend::ast::visitor::NodeVisitor;
use crate::mig::m4f::{AsyncContextFlags, ASYNC_CONTEXT_HAS_LEXICAL_THIS, ASYNC_CONTEXT_NON_TOP_LEVEL};

pub struct AsyncTransformer {
    pub context_flags: AsyncContextFlags,
    pub captured_super_properties: Option<HashSet<String>>,
    pub lexical_arguments: LexicalArguments,
    pub enclosing_function_parameter_names: Option<HashSet<String>>,
    pub emit_context: EmitContext,
}

pub struct LexicalArguments {
    pub binding: Option<Arc<Node>>,
    pub used: bool,
}

impl AsyncTransformer {
    fn emit_context(&self) -> &EmitContext { ::tsox_core::fntrace::enter("emit_context"); 
        &self.emit_context
    }

    fn emit_context_mut(&mut self) -> &mut EmitContext { ::tsox_core::fntrace::enter("emit_context_mut"); 
        &mut self.emit_context
    }

    fn factory(&self) -> NodeFactory<'_> { ::tsox_core::fntrace::enter("factory"); 
        NodeFactory::new(&self.emit_context)
    }

    fn visitor(&self) -> Visitor { ::tsox_core::fntrace::enter("visitor"); 
        Visitor::default()
    }

    fn fallback_node_visitor(&self) -> NodeVisitor { ::tsox_core::fntrace::enter("fallback_node_visitor"); 
        NodeVisitor::default()
    }

    fn set_context_flag(&mut self, flag: AsyncContextFlags, val: bool) { ::tsox_core::fntrace::enter("set_context_flag"); 
        if val {
            self.context_flags |= flag;
        } else {
            self.context_flags &= !flag;
        }
    }

    pub fn in_context(&self, flags: AsyncContextFlags) -> bool { ::tsox_core::fntrace::enter("in_context"); 
        self.context_flags & flags != 0
    }

    pub fn in_top_level_context(&self) -> bool { ::tsox_core::fntrace::enter("in_top_level_context"); 
        !self.in_context(ASYNC_CONTEXT_NON_TOP_LEVEL)
    }

    pub fn in_has_lexical_this_context(&self) -> bool { ::tsox_core::fntrace::enter("in_has_lexical_this_context"); 
        self.in_context(ASYNC_CONTEXT_HAS_LEXICAL_THIS)
    }

    pub fn doWithContext(
        &mut self,
        flags: AsyncContextFlags,
        cb: impl FnOnce(&mut Self, Option<Arc<Node>>) -> Option<Arc<Node>>,
        node: Option<Arc<Node>>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("doWithContext"); 
        let flags_to_set = flags & !self.context_flags;
        if flags_to_set != 0 {
            self.set_context_flag(flags_to_set, true);
            let result = cb(self, node);
            self.set_context_flag(flags_to_set, false);
            result
        } else {
            cb(self, node)
        }
    }

    fn track_super_access(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("track_super_access"); 
        let Some(captured) = self.captured_super_properties.as_mut() else {
            return;
        };
        match node.kind {
            SyntaxKind::PropertyAccessExpression => {
                if node
                    .expression()
                    .map(|e| e.kind == SyntaxKind::SuperKeyword)
                    .unwrap_or(false)
                {
                    if let Some(name) = node.name() {
                        captured.insert(name.text().to_string());
                    }
                }
            }
            _ => {}
        }
    }

    pub fn fallback_visitor(&mut self, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("fallback_visitor"); 
        if self.captured_super_properties.is_none() && self.lexical_arguments.binding.is_none() {
            return Some(node.clone());
        }
        self.track_super_access(&node);
        match node.kind {
            SyntaxKind::FunctionExpression
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::Constructor => return Some(node),
            SyntaxKind::Parameter | SyntaxKind::BindingElement | SyntaxKind::VariableDeclaration => {}
            SyntaxKind::Identifier => {
                if self.lexical_arguments.binding.is_some()
                    && node.text() == "arguments"
                    && !is_identifier_name(&node)
                    && !is_label_name(&node)
                {
                    self.lexical_arguments.used = true;
                    return self.lexical_arguments.binding.clone();
                }
            }
            _ => {}
        }
        self.fallback_node_visitor().visit_each_child(&node)
    }

    pub fn is_variable_declaration_list_with_colliding_name(&self, node: Option<&Arc<Node>>) -> bool { ::tsox_core::fntrace::enter("is_variable_declaration_list_with_colliding_name"); 
        match node {
            None => false,
            Some(node) => {
                is_variable_declaration_list(node)
                    && node.flags & NodeFlags::BlockScoped == NodeFlags::empty()
                    && crate::mig::m4e::r39k01_defs::R39K01DataExt::as_variable_declaration_list(&**node)
                        .declarations
                        .nodes
                        .iter()
                        .any(|decl| self.collides_with_parameter_name(decl))
            }
        }
    }

    pub fn hoist_variable_declaration_list(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("hoist_variable_declaration_list"); 
        for decl in &crate::mig::m4e::r39k01_defs::R39K01DataExt::as_variable_declaration_list(&**node)
            .declarations
            .nodes
        {
            self.hoist_variable(decl);
        }
    }

    pub fn hoist_variable(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("hoist_variable"); 
        let name = match node.name() {
            None => return,
            Some(name) => name,
        };
        if is_identifier(&name) {
            self.emit_context_mut().add_variable_declaration(&name);
        } else if is_binding_pattern(&name) {
            for element in &crate::mig::m4e::r37k1_defs::R37K1DataExt::as_binding_pattern(&**name)
                .elements
                .nodes
            {
                if !is_omitted_expression(element) {
                    self.hoist_variable(element);
                }
            }
        }
    }

    pub fn collides_with_parameter_name(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("collides_with_parameter_name"); 
        let name = match node.name() {
            None => return false,
            Some(name) => name,
        };
        if is_identifier(&name) {
            return self
                .enclosing_function_parameter_names
                .as_ref()
                .map(|names| names.contains(name.text()))
                .unwrap_or(false);
        }
        if is_binding_pattern(&name) {
            for element in &crate::mig::m4e::r37k1_defs::R37K1DataExt::as_binding_pattern(&**name)
                .elements
                .nodes
            {
                if !is_omitted_expression(element) && self.collides_with_parameter_name(element) {
                    return true;
                }
            }
        }
        false
    }

    pub fn create_capture_arguments_statement(&mut self) -> Arc<Node> { ::tsox_core::fntrace::enter("create_capture_arguments_statement"); 
        let name = self.lexical_arguments.binding.clone().unwrap();
        let init = self.factory().new_identifier("arguments");
        let variable = self.factory().new_variable_declaration(&name, None, None, Some(&init));
        let list = self.factory().new_node_list(vec![variable]);
        let decl_list = self.factory().new_variable_declaration_list(&list, NodeFlags::empty());
        let statement = self.factory().new_variable_statement(None, &decl_list);
        self.emit_context()
            .add_emit_flags(&statement, EmitFlags::START_ON_NEW_LINE | EmitFlags::CUSTOM_PROLOGUE);
        statement
    }

    pub fn get_original_if_function_like(&self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("get_original_if_function_like"); 
        let original = self.emit_context().most_original(node);
        if is_function_like_declaration(&original) {
            return original;
        }
        node.clone()
    }
}

pub fn assignment_target_contains_super_property(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("assignment_target_contains_super_property"); 
    match node.kind {
        SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression => {
            node.expression().map(|e| e.kind == SyntaxKind::SuperKeyword).unwrap_or(false)
        }
        SyntaxKind::ParenthesizedExpression => {
            assignment_target_contains_super_property(&node.as_parenthesized_expression().expression)
        }
        SyntaxKind::ArrayLiteralExpression => node
            .as_array_literal_expression()
            .elements
            .nodes
            .iter()
            .any(assignment_target_contains_super_property),
        SyntaxKind::ObjectLiteralExpression => {
            for prop in &node.as_object_literal_expression().properties.nodes {
                match prop.kind {
                    SyntaxKind::PropertyAssignment => {
                        if assignment_target_contains_super_property(
                            &crate::mig::m4q_3::r36k15_defs::NodeDataExt15::as_property_assignment(&**prop)
                                .initializer,
                        ) {
                            return true;
                        }
                    }
                    SyntaxKind::ShorthandPropertyAssignment => {
                        if prop.name().map(assignment_target_contains_super_property).unwrap_or(false)
                        {
                            return true;
                        }
                    }
                    SyntaxKind::SpreadAssignment => {
                        if assignment_target_contains_super_property(
                            &crate::mig::m4q_3::r36k15_defs::NodeDataExt15::as_spread_assignment(&**prop)
                                .expression,
                        ) {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
            false
        }
        SyntaxKind::SpreadElement => {
            assignment_target_contains_super_property(&node.as_spread_element().expression)
        }
        _ => false,
    }
}

pub fn is_update_expression(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_update_expression"); 
    if is_prefix_unary_expression(node) {
        let op = node.as_prefix_unary_expression().operator;
        return op == SyntaxKind::PlusPlusToken || op == SyntaxKind::MinusMinusToken;
    }
    if is_postfix_unary_expression(node) {
        let op = node.as_postfix_unary_expression().operator;
        return op == SyntaxKind::PlusPlusToken || op == SyntaxKind::MinusMinusToken;
    }
    false
}

pub fn is_simple_parameter_list(params: &[Arc<Node>]) -> bool { ::tsox_core::fntrace::enter("is_simple_parameter_list"); 
    for param in params {
        let p = crate::mig::m4e::r37k1_defs::R37K1DataExt::as_parameter_declaration(&**param);
        if p.initializer.is_some() || !param.name().map(|n| is_identifier(n)).unwrap_or(false) {
            return false;
        }
    }
    true
}

pub fn is_node_with_possible_hoisted_declaration(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_node_with_possible_hoisted_declaration"); 
    matches!(
        node.kind,
        SyntaxKind::Block
            | SyntaxKind::VariableStatement
            | SyntaxKind::WithStatement
            | SyntaxKind::IfStatement
            | SyntaxKind::SwitchStatement
            | SyntaxKind::CaseBlock
            | SyntaxKind::CaseClause
            | SyntaxKind::DefaultClause
            | SyntaxKind::LabeledStatement
            | SyntaxKind::ForStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
            | SyntaxKind::DoStatement
            | SyntaxKind::WhileStatement
            | SyntaxKind::TryStatement
            | SyntaxKind::CatchClause
    )
}
