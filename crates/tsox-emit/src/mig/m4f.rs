#![allow(unused_imports)]

use std::sync::Arc;

use tsox_core::collections::ordered_set::OrderedSet;
use tsox_core::collections::set::Set;
use tsox_frontend::ast::node::{Node, NodeList};
use tsox_frontend::ast::node_data_generated::{
    NodeData, is_assignment_operator, is_binding_pattern, is_identifier, is_omitted_expression,
};
use tsox_frontend::ast::subtree_facts::{
    SubtreeContainsAnyAwait, SubtreeContainsAwait, SubtreeFacts,
};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::visitor::NodeVisitor;
use tsox_frontend::format::mig::m4o::EmitFlags;

use crate::mig::m3m::TransformOptions;
use crate::mig::m4e_3::{
    assignment_target_contains_super_property, is_node_with_possible_hoisted_declaration,
    is_update_expression,
};
use crate::mig::m4k_2::Transformer;
use crate::printer::{AutoGenerateOptions, EmitContext, GeneratedIdentifierFlags, NodeFactory};
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::is_variable_declaration_list;

#[path = "r33k11_defs.rs"]
pub mod r33k11_defs;

#[path = "r39k19_defs.rs"]
pub mod r39k19_defs;

use r39k19_defs::R39K19NodeVisitorExt;
use crate::mig::m4f_2::r38k5_defs::R38K5NodeVisitorExt;
use crate::mig::m4g::r39k15_defs::NodeFactoryR39k15;
use crate::mig::m4k::R39K02NodeExt;
use tsox_frontend::ast::mig::m3g::is_label_name;

fn is_identifier_name(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_identifier_name"); 
    let Some(parent) = node.parent() else {
        return false;
    };
    match parent.kind {
        SyntaxKind::PropertyDeclaration
        | SyntaxKind::PropertySignature
        | SyntaxKind::MethodDeclaration
        | SyntaxKind::MethodSignature
        | SyntaxKind::GetAccessor
        | SyntaxKind::SetAccessor
        | SyntaxKind::EnumMember
        | SyntaxKind::PropertyAssignment
        | SyntaxKind::PropertyAccessExpression => {
            parent.name().is_some_and(|n| Arc::ptr_eq(n, node))
        }
        SyntaxKind::QualifiedName => {
            matches!(&parent.data, NodeData::QualifiedName(q) if Arc::ptr_eq(&q.right, node))
        }
        SyntaxKind::BindingElement => {
            matches!(&parent.data, NodeData::BindingElement(b) if b
                .property_name
                .as_ref()
                .is_some_and(|n| Arc::ptr_eq(n, node)))
        }
        SyntaxKind::ImportSpecifier => {
            matches!(&parent.data, NodeData::ImportSpecifier(i) if i
                .property_name
                .as_ref()
                .is_some_and(|n| Arc::ptr_eq(n, node)))
        }
        SyntaxKind::ExportSpecifier
        | SyntaxKind::JsxAttribute
        | SyntaxKind::JsxSelfClosingElement
        | SyntaxKind::JsxOpeningElement
        | SyntaxKind::JsxClosingElement => true,
        _ => false,
    }
}

pub type AsyncContextFlags = u32;

pub const ASYNC_CONTEXT_NON_TOP_LEVEL: AsyncContextFlags = 1 << 0;
pub const ASYNC_CONTEXT_HAS_LEXICAL_THIS: AsyncContextFlags = 1 << 1;

pub struct LexicalArgumentsInfo {
    pub binding: Option<Arc<Node>>,
    pub used: bool,
}

impl Default for LexicalArgumentsInfo {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        LexicalArgumentsInfo {
            binding: None,
            used: false,
        }
    }
}

pub struct SuperAccessState {
    pub captured_super_properties: Option<OrderedSet<String>>,
    pub has_super_element_access: bool,
    pub has_super_property_assignment: bool,
    pub super_binding: Option<Arc<Node>>,
    pub super_index_binding: Option<Arc<Node>>,
}

impl Default for SuperAccessState {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        SuperAccessState {
            captured_super_properties: None,
            has_super_element_access: false,
            has_super_property_assignment: false,
            super_binding: None,
            super_index_binding: None,
        }
    }
}

pub(crate) fn save_super_access_state(state: &mut SuperAccessState) -> SuperAccessState { ::tsox_core::fntrace::enter("save_super_access_state"); 
    std::mem::replace(
        state,
        SuperAccessState {
            captured_super_properties: None,
            has_super_element_access: false,
            has_super_property_assignment: false,
            super_binding: None,
            super_index_binding: None,
        },
    )
}

pub(crate) fn reset_super_access_state(tx: &mut AsyncTransformer, with_bindings: bool) { ::tsox_core::fntrace::enter("reset_super_access_state"); 
    tx.super_access.captured_super_properties = Some(OrderedSet::new());
    tx.super_access.has_super_element_access = false;
    tx.super_access.has_super_property_assignment = false;
    if with_bindings {
        let super_binding = tx.factory().generated_name_node(&tx.factory().new_unique_name_ex(
            "_super",
            AutoGenerateOptions {
                flags: GeneratedIdentifierFlags::OPTIMISTIC | GeneratedIdentifierFlags::FILE_LEVEL,
                ..Default::default()
            },
        ));
        tx.super_access.super_binding = Some(super_binding);
        let super_index_binding = tx.factory().generated_name_node(
            &tx.factory().new_unique_name_ex(
                "_superIndex",
                AutoGenerateOptions {
                    flags: GeneratedIdentifierFlags::OPTIMISTIC
                        | GeneratedIdentifierFlags::FILE_LEVEL,
                    ..Default::default()
                },
            ),
        );
        tx.super_access.super_index_binding = Some(super_index_binding);
    }
}

pub(crate) fn restore_super_access_state(state: &mut SuperAccessState, saved: SuperAccessState) { ::tsox_core::fntrace::enter("restore_super_access_state"); 
    *state = saved;
}

pub struct AsyncTransformer {
    pub super_access: SuperAccessState,
    pub context_flags: AsyncContextFlags,
    pub enclosing_function_parameter_names: Option<Set<String>>,
    pub lexical_arguments: LexicalArgumentsInfo,
    pub async_body_visitor: Option<NodeVisitor>,
    pub fallback_node_visitor: Option<NodeVisitor>,
    pub super_access_visitor: Option<NodeVisitor>,
    pub emit_context: Option<EmitContext>,
}

impl Default for AsyncTransformer {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        AsyncTransformer {
            super_access: SuperAccessState::default(),
            context_flags: 0,
            enclosing_function_parameter_names: None,
            lexical_arguments: LexicalArgumentsInfo::default(),
            async_body_visitor: None,
            fallback_node_visitor: None,
            super_access_visitor: None,
            emit_context: None,
        }
    }
}

impl AsyncTransformer {
    pub fn emit_context(&self) -> &EmitContext { ::tsox_core::fntrace::enter("emit_context"); 
        self.emit_context.as_ref().unwrap()
    }

    pub fn factory(&self) -> NodeFactory<'_> { ::tsox_core::fntrace::enter("factory"); 
        NodeFactory::new(self.emit_context())
    }
}

impl AsyncTransformer {
    pub fn new_async_transformer(opts: &TransformOptions) -> Arc<Transformer> { ::tsox_core::fntrace::enter("new_async_transformer"); 
        let mut tx = Box::<AsyncTransformer>::default();
        tx.emit_context = Some(opts.context.clone());
        let result =
            Arc::new(Transformer::new(r39k19_defs::async_transformer_visit_entry, Some(opts.context.clone())));
        tx.init_super_access_visitor();
        tx.async_body_visitor = Some(
            tx.emit_context()
                .new_node_visitor(|tx: &mut AsyncTransformer, node| tx.visit_async_body_node(&node)),
        );
        tx.fallback_node_visitor = Some(
            tx.emit_context()
                .new_node_visitor(|tx: &mut AsyncTransformer, node| tx.visit_fallback(&node)),
        );
        result
    }

    pub fn visit_source_file(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_source_file"); 
        if node.is_declaration_file_node() {
            return Some(Arc::clone(node));
        }
        self.set_context_flag(ASYNC_CONTEXT_NON_TOP_LEVEL, false);
        self.set_context_flag(ASYNC_CONTEXT_HAS_LEXICAL_THIS, false);
        let visited = self
            .visitor()
            .visit_each_child(node)
            .unwrap_or_else(|| Arc::clone(node));
        let helpers = self.emit_context_mut().read_emit_helpers();
        self.emit_context_mut().add_emit_helper(&visited, &helpers);
        Some(visited)
    }

    pub fn set_context_flag(&mut self, flag: AsyncContextFlags, val: bool) { ::tsox_core::fntrace::enter("set_context_flag"); 
        if val {
            self.context_flags |= flag;
        } else {
            self.context_flags &= !flag;
        }
    }

    pub fn do_with_context(
        &mut self,
        flags: AsyncContextFlags,
        cb: fn(&mut AsyncTransformer, &Arc<Node>) -> Option<Arc<Node>>,
        node: &Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("do_with_context"); 
        let flags_to_set = flags & !self.context_flags;
        if flags_to_set != 0 {
            self.set_context_flag(flags_to_set, true);
            let result = cb(self, node);
            self.set_context_flag(flags_to_set, false);
            return result;
        }
        cb(self, node)
    }

    pub fn visit_default(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_default"); 
        self.visitor().visit_each_child(node)
    }

    pub fn visit_fallback(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_fallback"); 
        self.fallback_visitor(node)
    }

    pub fn fallback_visitor(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("fallback_visitor"); 
        if self.super_access.captured_super_properties.is_none()
            && self.lexical_arguments.binding.is_none()
        {
            return Some(Arc::clone(node));
        }
        self.track_super_access(node);
        match node.kind {
            SyntaxKind::FunctionExpression
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::Constructor => Some(Arc::clone(node)),
            SyntaxKind::Identifier => {
                if self.lexical_arguments.binding.is_some()
                    && node.text() == "arguments"
                    && !is_identifier_name(node)
                    && !is_label_name(node)
                {
                    self.lexical_arguments.used = true;
                    return self.lexical_arguments.binding.clone();
                }
                self.fallback_node_visitor
                    .as_mut()
                    .expect("fallbackNodeVisitor")
                    .visit_each_child(node)
            }
            _ => self
                .fallback_node_visitor
                .as_mut()
                .expect("fallbackNodeVisitor")
                .visit_each_child(node),
        }
    }

    pub fn visit(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit"); 
        let restore_lexical_this =
            self.emit_context().emit_flags(node).contains(EmitFlags::NO_LEXICAL_THIS)
                && self.in_has_lexical_this_context();
        if restore_lexical_this {
            self.set_context_flag(ASYNC_CONTEXT_HAS_LEXICAL_THIS, false);
        }
        let result = self.visit_inner(node);
        if restore_lexical_this {
            self.set_context_flag(ASYNC_CONTEXT_HAS_LEXICAL_THIS, true);
        }
        result
    }

    fn visit_inner(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_inner"); 
        if !node
            .subtree_facts()
            .intersects(SubtreeContainsAnyAwait | SubtreeContainsAwait)
        {
            return self.fallback_visitor(node);
        }
        self.track_super_access(node);
        match node.kind {
            SyntaxKind::AsyncKeyword => None,
            SyntaxKind::SourceFile => self.visit_source_file(node),
            SyntaxKind::AwaitExpression => self.visit_await_expression(node),
            SyntaxKind::MethodDeclaration => self.do_with_context(
                ASYNC_CONTEXT_NON_TOP_LEVEL | ASYNC_CONTEXT_HAS_LEXICAL_THIS,
                AsyncTransformer::visit_method_declaration,
                node,
            ),
            SyntaxKind::FunctionDeclaration => self.do_with_context(
                ASYNC_CONTEXT_NON_TOP_LEVEL | ASYNC_CONTEXT_HAS_LEXICAL_THIS,
                AsyncTransformer::visit_function_declaration,
                node,
            ),
            SyntaxKind::FunctionExpression => self.do_with_context(
                ASYNC_CONTEXT_NON_TOP_LEVEL | ASYNC_CONTEXT_HAS_LEXICAL_THIS,
                AsyncTransformer::visit_function_expression,
                node,
            ),
            SyntaxKind::ArrowFunction => {
                self.do_with_context(ASYNC_CONTEXT_NON_TOP_LEVEL, AsyncTransformer::visit_arrow_function, node)
            }
            SyntaxKind::GetAccessor => self.do_with_context(
                ASYNC_CONTEXT_NON_TOP_LEVEL | ASYNC_CONTEXT_HAS_LEXICAL_THIS,
                AsyncTransformer::visit_get_accessor_declaration,
                node,
            ),
            SyntaxKind::SetAccessor => self.do_with_context(
                ASYNC_CONTEXT_NON_TOP_LEVEL | ASYNC_CONTEXT_HAS_LEXICAL_THIS,
                AsyncTransformer::visit_set_accessor_declaration,
                node,
            ),
            SyntaxKind::Constructor => self.do_with_context(
                ASYNC_CONTEXT_NON_TOP_LEVEL | ASYNC_CONTEXT_HAS_LEXICAL_THIS,
                AsyncTransformer::visit_constructor_declaration,
                node,
            ),
            SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => self.do_with_context(
                ASYNC_CONTEXT_NON_TOP_LEVEL | ASYNC_CONTEXT_HAS_LEXICAL_THIS,
                AsyncTransformer::visit_default,
                node,
            ),
            _ => self.visitor().visit_each_child(node),
        }
    }

    pub fn visit_async_body_node(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_async_body_node"); 
        if is_node_with_possible_hoisted_declaration(node) {
            match node.kind {
                SyntaxKind::VariableStatement => {
                    return self.visit_variable_statement_in_async_body(node)
                }
                SyntaxKind::ForStatement => {
                    return self.visit_for_statement_in_async_body(node)
                }
                SyntaxKind::ForInStatement => {
                    return self.visit_for_in_statement_in_async_body(node)
                }
                SyntaxKind::ForOfStatement => {
                    return self.visit_for_of_statement_in_async_body(node)
                }
                SyntaxKind::CatchClause => {
                    return self.visit_catch_clause_in_async_body(node)
                }
                SyntaxKind::Block
                | SyntaxKind::SwitchStatement
                | SyntaxKind::CaseBlock
                | SyntaxKind::CaseClause
                | SyntaxKind::DefaultClause
                | SyntaxKind::TryStatement
                | SyntaxKind::DoStatement
                | SyntaxKind::WhileStatement
                | SyntaxKind::IfStatement
                | SyntaxKind::WithStatement
                | SyntaxKind::LabeledStatement => {
                    return self
                        .async_body_visitor
                        .as_mut()
                        .and_then(|v| v.visit_each_child(node))
                }
                _ => {}
            }
        }
        self.visit(node)
    }

    pub fn visit_catch_clause_in_async_body(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_catch_clause_in_async_body"); 
        let mut catch_clause_names = Set::<String>::new();
        if let NodeData::CatchClause(d) = &node.data {
            if let Some(variable_declaration) = &d.variable_declaration {
                self.record_declaration_name(variable_declaration, &mut catch_clause_names);
            }
        }

        let mut catch_clause_unshadowed_names: Option<Set<String>> = None;
        for escaped_name in catch_clause_names.iter() {
            if let Some(enclosing) = &self.enclosing_function_parameter_names {
                if enclosing.has(escaped_name) {
                    let unshadowed = catch_clause_unshadowed_names
                        .get_or_insert_with(|| enclosing.clone());
                    unshadowed.delete(escaped_name);
                }
            }
        }

        if let Some(unshadowed) = catch_clause_unshadowed_names {
            let saved_enclosing_function_parameter_names =
                self.enclosing_function_parameter_names.replace(unshadowed);
            let result = self
                .async_body_visitor
                .as_mut()
                .and_then(|v| v.visit_each_child(node));
            self.enclosing_function_parameter_names = saved_enclosing_function_parameter_names;
            return result;
        }
        self.async_body_visitor
            .as_mut()
            .and_then(|v| v.visit_each_child(node))
    }

    pub fn visit_variable_statement_in_async_body(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_variable_statement_in_async_body"); 
        let decl_list = match &node.data {
            NodeData::VariableStatement(d) => Arc::clone(&d.declaration_list),
            _ => return self.visitor().visit_each_child(node),
        };
        if self.is_variable_declaration_list_with_colliding_name(&decl_list) {
            let expression = self.visit_variable_declaration_list_with_colliding_names(
                &decl_list,
                false,
            );
            if let Some(expression) = expression {
                return Some(self.factory().new_expression_statement(&expression));
            }
            return None;
        }
        self.visitor().visit_each_child(node)
    }

    pub fn visit_for_in_statement_in_async_body(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_for_in_statement_in_async_body"); 
        let (initializer, expression, statement) = match &node.data {
            NodeData::ForInOrOfStatement(d) => (
                Arc::clone(&d.initializer),
                Arc::clone(&d.expression),
                Arc::clone(&d.statement),
            ),
            _ => return self.visitor().visit_each_child(node),
        };
        let visited_initializer = if self.is_variable_declaration_list_with_colliding_name(&initializer) {
            self.visit_variable_declaration_list_with_colliding_names(&initializer, true)
        } else {
            Some(self.visitor().visit_node(&initializer))
        };
        let visited_expression = self.visitor().visit_node(&expression);
        let visited_statement = self
            .async_body_visitor
            .as_mut()
            .and_then(|v| v.visit_embedded_statement(&statement));
        let updated = self.factory().update_for_in_or_of_statement(
            node,
            None,
            visited_initializer,
            Some(visited_expression),
            visited_statement,
        );
        Some(updated)
    }

    pub fn visit_for_of_statement_in_async_body(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_for_of_statement_in_async_body"); 
        let (await_modifier, initializer, expression, statement) = match &node.data {
            NodeData::ForInOrOfStatement(d) => (
                d.await_modifier.clone(),
                Arc::clone(&d.initializer),
                Arc::clone(&d.expression),
                Arc::clone(&d.statement),
            ),
            _ => return self.visitor().visit_each_child(node),
        };
        let visited_initializer = if self.is_variable_declaration_list_with_colliding_name(&initializer) {
            self.visit_variable_declaration_list_with_colliding_names(&initializer, true)
        } else {
            Some(self.visitor().visit_node(&initializer))
        };
        let visited_await_modifier = await_modifier.map(|m| self.visitor().visit_node(&m));
        let visited_expression = self.visitor().visit_node(&expression);
        let visited_statement = self
            .async_body_visitor
            .as_mut()
            .and_then(|v| v.visit_embedded_statement(&statement));
        let updated = self.factory().update_for_in_or_of_statement(
            node,
            visited_await_modifier,
            visited_initializer,
            Some(visited_expression),
            visited_statement,
        );
        Some(updated)
    }

    pub fn visit_for_statement_in_async_body(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_for_statement_in_async_body"); 
        let (initializer, condition, incrementor, statement) = match &node.data {
            NodeData::ForStatement(d) => (
                d.initializer.clone(),
                d.condition.clone(),
                d.incrementor.clone(),
                Arc::clone(&d.statement),
            ),
            _ => return self.visitor().visit_each_child(node),
        };
        let visited_initializer = if initializer.is_some()
            && self.is_variable_declaration_list_with_colliding_name(initializer.as_ref().unwrap())
        {
            self.visit_variable_declaration_list_with_colliding_names(
                initializer.as_ref().unwrap(),
                false,
            )
        } else {
            self.visitor().visit_node_option(initializer.as_ref())
        };
        let visited_statement = self
            .async_body_visitor
            .as_mut()
            .and_then(|v| v.visit_embedded_statement(&statement))
            .unwrap_or_else(|| Arc::clone(&statement));
        let visited_condition = condition.map(|c| self.visitor().visit_node(&c));
        let visited_incrementor = incrementor.map(|i| self.visitor().visit_node(&i));
        let updated = self.factory().update_for_statement(
            node,
            visited_initializer.as_ref(),
            visited_condition.as_ref(),
            visited_incrementor.as_ref(),
            &visited_statement,
        );
        Some(updated)
    }

    pub fn visit_await_expression(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_await_expression"); 
        if self.in_top_level_context() {
            return self.visitor().visit_each_child(node);
        }
        let expression = match &node.data {
            NodeData::AwaitExpression(d) => Arc::clone(&d.expression),
            _ => return self.visitor().visit_each_child(node),
        };
        let visited_expression = self.visitor().visit_node(&expression);
        let mut yield_expr = self.factory().new_yield_expression(None, &visited_expression);
        if let Some(yield_expr) = Arc::get_mut(&mut yield_expr) {
            yield_expr.loc = node.loc;
        }
        self.emit_context().set_original(&yield_expr, node);
        Some(yield_expr)
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

    pub fn track_super_access(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("track_super_access"); 
        if self.super_access.captured_super_properties.is_none() {
            return;
        }
        match node.kind {
            SyntaxKind::PropertyAccessExpression => {
                if node.expression().unwrap().kind == SyntaxKind::SuperKeyword {
                    if let Some(name) = node.name() {
                        if let Some(captured) = self.super_access.captured_super_properties.as_mut()
                        {
                            captured.add(name.text().to_string());
                        }
                    }
                }
            }
            SyntaxKind::ElementAccessExpression => {
                if node.expression().unwrap().kind == SyntaxKind::SuperKeyword {
                    self.super_access.has_super_element_access = true;
                }
            }
            SyntaxKind::BinaryExpression => {
                if let NodeData::BinaryExpression(d) = &node.data {
                    if is_assignment_operator(d.operator_token.kind)
                        && assignment_target_contains_super_property(&d.left)
                    {
                        self.super_access.has_super_property_assignment = true;
                    }
                }
            }
            SyntaxKind::PrefixUnaryExpression => {
                if let NodeData::PrefixUnaryExpression(d) = &node.data {
                    if is_update_expression(node)
                        && assignment_target_contains_super_property(&d.operand)
                    {
                        self.super_access.has_super_property_assignment = true;
                    }
                }
            }
            SyntaxKind::PostfixUnaryExpression => {
                if let NodeData::PostfixUnaryExpression(d) = &node.data {
                    if is_update_expression(node)
                        && assignment_target_contains_super_property(&d.operand)
                    {
                        self.super_access.has_super_property_assignment = true;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn is_variable_declaration_list_with_colliding_name(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_variable_declaration_list_with_colliding_name"); 
        is_variable_declaration_list(node)
            && node.flags & NodeFlags::BlockScoped == NodeFlags::empty()
            && match &node.data {
                NodeData::VariableDeclarationList(d) => d
                    .declarations
                    .nodes
                    .iter()
                    .any(|decl| self.collides_with_parameter_name(decl)),
                _ => false,
            }
    }

    fn collides_with_parameter_name(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("collides_with_parameter_name"); 
        let name = match node.name() {
            Some(name) => name,
            None => return false,
        };
        if is_identifier(&name) {
            return self
                .enclosing_function_parameter_names
                .as_ref()
                .is_some_and(|names| names.has(&name.text().to_string()));
        }
        if is_binding_pattern(&name) {
            if let NodeData::BindingPattern(d) = &name.data {
                for element in &d.elements.nodes {
                    if !is_omitted_expression(element) && self.collides_with_parameter_name(element)
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn record_declaration_name(&mut self, node: &Arc<Node>, names: &mut Set<String>) { ::tsox_core::fntrace::enter("record_declaration_name"); 
        let name = node.name();
        let name = match name {
            Some(name) => name,
            None => return,
        };
        if is_identifier(&name) {
            names.add(name.text().to_string());
        } else if is_binding_pattern(&name) {
            if let NodeData::BindingPattern(d) = &name.data {
                for element in &d.elements.nodes {
                    if !is_omitted_expression(element) {
                        self.record_declaration_name(element, names);
                    }
                }
            }
        }
    }
}
