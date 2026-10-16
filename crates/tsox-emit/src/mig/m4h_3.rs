use std::sync::Arc;
use tsox_frontend::ast::*;

use crate::mig::m3m::TransformOptions;
use crate::mig::m4f::SuperAccessState;
use crate::mig::m4k_2::Transformer;
use crate::printer::{EmitContext, NodeFactory as Factory};
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_frontend::ast::mig::m3e::FunctionFlags;
use tsox_frontend::ast::subtree_facts::SubtreeFacts;
use tsox_frontend::ast::mig::m3g_2::is_super_property;
use tsox_frontend::ast::{is_assignment_operator, is_element_access_expression, is_property_access_expression};
use crate::mig::m4e_3::{assignment_target_contains_super_property, is_update_expression};
use tsox_frontend::ast::visitor::NodeVisitor;
use super::m4h_4::r36k9_defs::FunctionFlagsExt;
use crate::mig::m4h::r37k18_defs::R37K18NodeVisitorExt;
use crate::mig::m4h::r39k13_defs;
use crate::mig::m4h::r39k13_defs::R39K13NodeVisitorExt;
use crate::mig::m4i_12::r36k17_defs::NodeAsR36k17Ext;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct ForAwaitHierarchyFacts(pub u32);

impl ForAwaitHierarchyFacts {
    pub const NONE: ForAwaitHierarchyFacts = ForAwaitHierarchyFacts(0);
    pub const HAS_LEXICAL_THIS: ForAwaitHierarchyFacts = ForAwaitHierarchyFacts(1 << 0);
    pub const ITERATION_CONTAINER: ForAwaitHierarchyFacts = ForAwaitHierarchyFacts(1 << 1);
    pub const ANCESTOR_FACTS_MASK: ForAwaitHierarchyFacts = ForAwaitHierarchyFacts((1 << 2) - 1);
    pub const SOURCE_FILE_EXCLUDES: ForAwaitHierarchyFacts = ForAwaitHierarchyFacts::ITERATION_CONTAINER;
    pub const STRICT_MODE_SOURCE_FILE_INCLUDES: ForAwaitHierarchyFacts = ForAwaitHierarchyFacts::NONE;
    pub const CLASS_OR_FUNCTION_INCLUDES: ForAwaitHierarchyFacts = ForAwaitHierarchyFacts::HAS_LEXICAL_THIS;
    pub const CLASS_OR_FUNCTION_EXCLUDES: ForAwaitHierarchyFacts = ForAwaitHierarchyFacts::ITERATION_CONTAINER;
    pub const ARROW_FUNCTION_INCLUDES: ForAwaitHierarchyFacts = ForAwaitHierarchyFacts::NONE;
    pub const ARROW_FUNCTION_EXCLUDES: ForAwaitHierarchyFacts = ForAwaitHierarchyFacts::CLASS_OR_FUNCTION_EXCLUDES;
    pub const ITERATION_STATEMENT_INCLUDES: ForAwaitHierarchyFacts = ForAwaitHierarchyFacts::ITERATION_CONTAINER;
    pub const ITERATION_STATEMENT_EXCLUDES: ForAwaitHierarchyFacts = ForAwaitHierarchyFacts::NONE;

    pub fn union(self, other: ForAwaitHierarchyFacts) -> ForAwaitHierarchyFacts { ::tsox_core::fntrace::enter("union"); 
        ForAwaitHierarchyFacts(self.0 | other.0)
    }

    pub fn minus(self, other: ForAwaitHierarchyFacts) -> ForAwaitHierarchyFacts { ::tsox_core::fntrace::enter("minus"); 
        ForAwaitHierarchyFacts(self.0 & !other.0)
    }

    pub fn intersection(self, other: ForAwaitHierarchyFacts) -> ForAwaitHierarchyFacts { ::tsox_core::fntrace::enter("intersection"); 
        ForAwaitHierarchyFacts(self.0 & other.0)
    }

    pub fn intersects(self, other: ForAwaitHierarchyFacts) -> bool { ::tsox_core::fntrace::enter("intersects"); 
        self.0 & other.0 != 0
    }
}

pub struct ForAwaitTransformer {
    pub transformer: Transformer,
    pub super_access_state: SuperAccessState,
    pub compiler_options: CompilerOptions,

    pub enclosing_function_flags: FunctionFlags,
    pub for_await_hierarchy_facts: ForAwaitHierarchyFacts,
    pub exported_variable_statement: bool,

    pub fallback_node_visitor: NodeVisitor,
    pub no_async_modifier_visitor: NodeVisitor,
    pub super_access_visitor: Option<NodeVisitor>,
}

pub fn new_for_await_transformer(opts: &TransformOptions) -> Transformer { ::tsox_core::fntrace::enter("new_for_await_transformer"); 
    let mut tx = ForAwaitTransformer {
        transformer: r39k13_defs::placeholder_transformer(),
        super_access_state: SuperAccessState::default(),
        compiler_options: opts.compiler_options.clone(),
        enclosing_function_flags: FunctionFlags::NORMAL,
        for_await_hierarchy_facts: ForAwaitHierarchyFacts::NONE,
        exported_variable_statement: false,
        fallback_node_visitor: NodeVisitor::default(),
        no_async_modifier_visitor: NodeVisitor::default(),
        super_access_visitor: None,
    };
    let result = Transformer::new(for_await_transformer_visit, Some(opts.context.clone()));
    tx.init_super_access_visitor(&result.emit_context(), &result.factory());
    tx.fallback_node_visitor = tx
        .transformer
        .emit_context()
        .new_node_visitor(|tx: &mut ForAwaitTransformer, node| {
            Some(tx.visit_fallback(&node))
        });
    tx.no_async_modifier_visitor = tx
        .transformer
        .emit_context()
        .new_node_visitor(|_: &mut ForAwaitTransformer, node: Arc<Node>| {
            if node.kind == SyntaxKind::AsyncKeyword {
                None
            } else {
                Some(node)
            }
        });
    result
}

fn for_await_transformer_visit(tx: &mut Transformer, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("for_await_transformer_visit"); 
    let _ = tx;
    Some(node)
}

impl ForAwaitTransformer {
    pub fn affects_subtree(
        &self,
        exclude_facts: ForAwaitHierarchyFacts,
        include_facts: ForAwaitHierarchyFacts,
    ) -> bool { ::tsox_core::fntrace::enter("affects_subtree"); 
        self.for_await_hierarchy_facts
            != self
                .for_await_hierarchy_facts
                .minus(exclude_facts)
                .union(include_facts)
    }

    pub fn enter_subtree(
        &mut self,
        exclude_facts: ForAwaitHierarchyFacts,
        include_facts: ForAwaitHierarchyFacts,
    ) -> ForAwaitHierarchyFacts { ::tsox_core::fntrace::enter("enter_subtree"); 
        let ancestor_facts = self.for_await_hierarchy_facts;
        self.for_await_hierarchy_facts = self
            .for_await_hierarchy_facts
            .minus(exclude_facts)
            .union(include_facts)
            .intersection(ForAwaitHierarchyFacts::ANCESTOR_FACTS_MASK);
        ancestor_facts
    }

    pub fn exit_subtree(&mut self, ancestor_facts: ForAwaitHierarchyFacts) { ::tsox_core::fntrace::enter("exit_subtree"); 
        self.for_await_hierarchy_facts = ancestor_facts;
    }

    pub fn visit_modifiers_no_async(&mut self, modifiers: &Option<Arc<ModifierList>>) -> Option<Arc<ModifierList>> { ::tsox_core::fntrace::enter("visit_modifiers_no_async"); 
        self.no_async_modifier_visitor.visit_modifiers(modifiers)
    }

    pub fn do_with_hierarchy_facts(
        &mut self,
        cb: impl FnOnce(&mut Self, &Arc<Node>) -> Arc<Node>,
        node: &Arc<Node>,
        exclude_facts: ForAwaitHierarchyFacts,
        include_facts: ForAwaitHierarchyFacts,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("do_with_hierarchy_facts"); 
        if self.affects_subtree(exclude_facts, include_facts) {
            let ancestor_facts = self.enter_subtree(exclude_facts, include_facts);
            let result = cb(self, node);
            self.exit_subtree(ancestor_facts);
            return result;
        }
        cb(self, node)
    }

    pub fn visit_default(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_default"); 
        self.transformer.visitor().visit_each_child(node)
    }

    pub fn fallback_visitor(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("fallback_visitor"); 
        if self.super_access_state.captured_super_properties.is_none() {
            return node.clone();
        }
        match node.kind {
            SyntaxKind::FunctionExpression
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::Constructor => return node.clone(),
            _ => {}
        }
        self.track_super_access(node);
        self.fallback_node_visitor.visit_each_child(node)
    }

    pub fn visit_fallback(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_fallback"); 
        self.fallback_visitor(node)
    }

    pub fn visit(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit"); 
        if !node
            .subtree_facts()
            .intersects(SubtreeFacts::CONTAINS_FOR_AWAIT_OR_ASYNC_GENERATOR)
        {
            return self.fallback_visitor(node);
        }
        self.track_super_access(node);
        match node.kind {
            SyntaxKind::SourceFile => self.visit_source_file(node),
            SyntaxKind::AwaitExpression => self.visit_await_expression(node),
            SyntaxKind::YieldExpression => self.visit_yield_expression(node),
            SyntaxKind::ReturnStatement => self.visit_return_statement(node),
            SyntaxKind::LabeledStatement => self.visit_labeled_statement(node),
            SyntaxKind::DoStatement | SyntaxKind::WhileStatement | SyntaxKind::ForInStatement => {
                self.do_with_hierarchy_facts(
                    Self::visit_default,
                    node,
                    ForAwaitHierarchyFacts::ITERATION_STATEMENT_EXCLUDES,
                    ForAwaitHierarchyFacts::ITERATION_STATEMENT_INCLUDES,
                )
            }
            SyntaxKind::ForOfStatement => self.visit_for_of_statement(node, None),
            SyntaxKind::ForStatement => self.do_with_hierarchy_facts(
                Self::visit_default,
                node,
                ForAwaitHierarchyFacts::ITERATION_STATEMENT_EXCLUDES,
                ForAwaitHierarchyFacts::ITERATION_STATEMENT_INCLUDES,
            ),
            SyntaxKind::Constructor => self.do_with_hierarchy_facts(
                Self::visit_constructor_declaration,
                node,
                ForAwaitHierarchyFacts::CLASS_OR_FUNCTION_EXCLUDES,
                ForAwaitHierarchyFacts::CLASS_OR_FUNCTION_INCLUDES,
            ),
            SyntaxKind::MethodDeclaration => self.do_with_hierarchy_facts(
                Self::visit_method_declaration,
                node,
                ForAwaitHierarchyFacts::CLASS_OR_FUNCTION_EXCLUDES,
                ForAwaitHierarchyFacts::CLASS_OR_FUNCTION_INCLUDES,
            ),
            SyntaxKind::GetAccessor => self.do_with_hierarchy_facts(
                Self::visit_get_accessor_declaration,
                node,
                ForAwaitHierarchyFacts::CLASS_OR_FUNCTION_EXCLUDES,
                ForAwaitHierarchyFacts::CLASS_OR_FUNCTION_INCLUDES,
            ),
            SyntaxKind::SetAccessor => self.do_with_hierarchy_facts(
                Self::visit_set_accessor_declaration,
                node,
                ForAwaitHierarchyFacts::CLASS_OR_FUNCTION_EXCLUDES,
                ForAwaitHierarchyFacts::CLASS_OR_FUNCTION_INCLUDES,
            ),
            SyntaxKind::FunctionDeclaration => self.do_with_hierarchy_facts(
                Self::visit_function_declaration,
                node,
                ForAwaitHierarchyFacts::CLASS_OR_FUNCTION_EXCLUDES,
                ForAwaitHierarchyFacts::CLASS_OR_FUNCTION_INCLUDES,
            ),
            SyntaxKind::FunctionExpression => self.do_with_hierarchy_facts(
                Self::visit_function_expression,
                node,
                ForAwaitHierarchyFacts::CLASS_OR_FUNCTION_EXCLUDES,
                ForAwaitHierarchyFacts::CLASS_OR_FUNCTION_INCLUDES,
            ),
            SyntaxKind::ArrowFunction => self.do_with_hierarchy_facts(
                Self::visit_arrow_function,
                node,
                ForAwaitHierarchyFacts::ARROW_FUNCTION_EXCLUDES,
                ForAwaitHierarchyFacts::ARROW_FUNCTION_INCLUDES,
            ),
            SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => self.do_with_hierarchy_facts(
                Self::visit_default,
                node,
                ForAwaitHierarchyFacts::CLASS_OR_FUNCTION_EXCLUDES,
                ForAwaitHierarchyFacts::CLASS_OR_FUNCTION_INCLUDES,
            ),
            _ => self.transformer.visitor().visit_each_child(node),
        }
    }

    pub fn visit_await_expression(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_await_expression"); 
        if self.enclosing_function_flags.0
            & (FunctionFlags::ASYNC.0 | FunctionFlags::GENERATOR.0)
            != 0
        {
            let expression = match &node.data {
                NodeData::AwaitExpression(d) => d.expression.clone(),
                _ => unreachable!(),
            };
            let visited = self.transformer.visitor().visit_node(&expression);
            let mut result = self.transformer.factory().new_yield_expression(
                None,
                &self.transformer.factory().new_await_helper(&visited),
            );
            r39k13_defs::set_loc(&mut result, node.loc);
            self.transformer.emit_context().set_original(&result, node);
            return result;
        }
        self.transformer.visitor().visit_each_child(node)
    }

    pub fn visit_yield_expression(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_yield_expression"); 
        if self.enclosing_function_flags.0
            & (FunctionFlags::ASYNC.0 | FunctionFlags::GENERATOR.0)
            != 0
        {
            let emit_context = self.transformer.emit_context();
            let (asterisk_token, expression) = match &node.data {
                NodeData::YieldExpression(d) => (d.asterisk_token.clone(), d.expression.clone()),
                _ => unreachable!(),
            };
            if asterisk_token.is_some() {
                let expression = expression.unwrap();
                let expression = self.transformer.visitor().visit_node(&expression);

                let factory = self.transformer.factory();
                let mut async_values_result = factory.new_async_values_helper(&expression);
                r39k13_defs::set_loc(&mut async_values_result, expression.loc);

                let mut async_delegator_result = factory.new_async_delegator_helper(&async_values_result);
                r39k13_defs::set_loc(&mut async_delegator_result, expression.loc);

                let inner_yield = factory.update_yield_expression(
                    node,
                    asterisk_token.as_ref(),
                    Some(&async_delegator_result),
                );

                let awaited_yield = factory.new_await_helper(&inner_yield);

                let mut result = factory.new_yield_expression(None, &awaited_yield);
                r39k13_defs::set_loc(&mut result, node.loc);
                emit_context.set_original(&result, node);
                return result;
            }

            let inner_expression = match expression {
                Some(expr) => self.transformer.visitor().visit_node(&expr),
                None => self.transformer.factory().new_void_zero_expression(),
            };

            let awaited_expression = self.create_downlevel_await(&inner_expression);
            let mut result = self.transformer.factory().new_yield_expression(None, &awaited_expression);
            r39k13_defs::set_loc(&mut result, node.loc);
            emit_context.set_original(&result, node);
            return result;
        }

        self.transformer.visitor().visit_each_child(node)
    }

    pub fn visit_return_statement(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_return_statement"); 
        if self.enclosing_function_flags.0
            & (FunctionFlags::ASYNC.0 | FunctionFlags::GENERATOR.0)
            != 0
        {
            let expression = match &node.data {
                NodeData::ReturnStatement(d) => d.expression.clone(),
                _ => unreachable!(),
            };
            let expression = match expression {
                Some(expr) => self.transformer.visitor().visit_node(&expr),
                None => self.transformer.factory().new_void_zero_expression(),
            };
            let awaited_expression = self.create_downlevel_await(&expression);
            return self
                .transformer
                .factory()
                .update_return_statement(node, Some(&awaited_expression));
        }

        self.transformer.visitor().visit_each_child(node)
    }

    pub fn visit_labeled_statement(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_labeled_statement"); 
        if self.enclosing_function_flags.intersects(FunctionFlags::ASYNC) {
            let statement = unwrap_innermost_statement_of_label(node);
            if statement.kind == SyntaxKind::ForOfStatement {
                let await_modifier = match &statement.data {
                    NodeData::ForInOrOfStatement(d) => d.await_modifier.clone(),
                    _ => None,
                };
                if await_modifier.is_some() {
                    return self.visit_for_of_statement(&statement, Some(node));
                }
            }
            let visited = self.transformer.visitor().visit_node(&statement);
            return self
                .transformer
                .factory()
                .restore_enclosing_label(&visited, Some(node));
        }
        self.transformer.visitor().visit_each_child(node)
    }

    pub fn visit_source_file(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_source_file"); 
        let ancestor_facts = self.enter_subtree(
            ForAwaitHierarchyFacts::SOURCE_FILE_EXCLUDES,
            ForAwaitHierarchyFacts::STRICT_MODE_SOURCE_FILE_INCLUDES,
        );
        self.exported_variable_statement = false;
        let visited = self.transformer.visitor().visit_each_child(node);
        self.transformer
            .emit_context()
            .add_emit_helper(&visited, &self.transformer.emit_context().read_emit_helpers());
        self.exit_subtree(ancestor_facts);
        visited
    }

    pub fn visit_for_of_statement(
        &mut self,
        node: &Arc<Node>,
        outermost_labeled_statement: Option<&Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_for_of_statement"); 
        let ancestor_facts = self.enter_subtree(
            ForAwaitHierarchyFacts::ITERATION_STATEMENT_EXCLUDES,
            ForAwaitHierarchyFacts::ITERATION_STATEMENT_INCLUDES,
        );
        let await_modifier = match &node.data {
            NodeData::ForInOrOfStatement(d) => d.await_modifier.clone(),
            _ => None,
        };
        let result = if await_modifier.is_some() {
            self.transform_for_await_of_statement(node, outermost_labeled_statement, ancestor_facts)
        } else {
            let visited = self.transformer.visitor().visit_each_child(node);
            self.transformer
                .factory()
                .restore_enclosing_label(&visited, outermost_labeled_statement)
        };
        self.exit_subtree(ancestor_facts);
        result
    }

    pub fn track_super_access(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("track_super_access"); 
        if self.super_access_state.captured_super_properties.is_none() {
            return;
        }
        match node.kind {
            SyntaxKind::PropertyAccessExpression => {
                if node.expression().unwrap().kind == SyntaxKind::SuperKeyword {
                    if let Some(name) = node.name() {
                        if let Some(captured) =
                            self.super_access_state.captured_super_properties.as_mut()
                        {
                            captured.add(name.text().to_string());
                        }
                    }
                }
            }
            SyntaxKind::ElementAccessExpression => {
                if node.expression().unwrap().kind == SyntaxKind::SuperKeyword {
                    self.super_access_state.has_super_element_access = true;
                }
            }
            SyntaxKind::BinaryExpression => {
                if let NodeData::BinaryExpression(d) = &node.data {
                    if is_assignment_operator(d.operator_token.kind)
                        && assignment_target_contains_super_property(&d.left)
                    {
                        self.super_access_state.has_super_property_assignment = true;
                    }
                }
            }
            SyntaxKind::PrefixUnaryExpression => {
                if let NodeData::PrefixUnaryExpression(d) = &node.data {
                    if is_update_expression(node)
                        && assignment_target_contains_super_property(&d.operand)
                    {
                        self.super_access_state.has_super_property_assignment = true;
                    }
                }
            }
            SyntaxKind::PostfixUnaryExpression => {
                if let NodeData::PostfixUnaryExpression(d) = &node.data {
                    if is_update_expression(node)
                        && assignment_target_contains_super_property(&d.operand)
                    {
                        self.super_access_state.has_super_property_assignment = true;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn init_super_access_visitor(&mut self, emit_context: &EmitContext, _factory: &Factory) { ::tsox_core::fntrace::enter("init_super_access_visitor"); 
        self.super_access_visitor =
            Some(emit_context.new_node_visitor(Self::visit_super_access_node));
    }

    pub fn substitute_super_accesses_in_body(&mut self, body: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("substitute_super_accesses_in_body"); 
        match self.super_access_visitor.as_mut() {
            Some(v) => v.visit_node(body),
            None => body.clone(),
        }
    }

    fn visit_super_access_node(&mut self, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_super_access_node"); 
        match node.kind {
            SyntaxKind::CallExpression => {
                let expression = node.as_call_expression().expression.clone();
                if is_super_property(&expression) {
                    return Some(self.substitute_call_expression_with_super_access(&node));
                }
                self.super_access_visitor
                    .as_mut()
                    .map(|v| v.visit_node(&node))
                    .unwrap_or_else(|| node.clone())
            }
            SyntaxKind::PropertyAccessExpression => {
                if node.expression().unwrap().kind == SyntaxKind::SuperKeyword {
                    let factory = self.transformer.factory();
                    let super_binding = self.super_access_state.super_binding.clone().unwrap();
                    return Some(factory.new_property_access_expression(
                        &super_binding,
                        None,
                        node.name().unwrap(),
                        NodeFlags::empty(),
                    ));
                }
                self.super_access_visitor
                    .as_mut()
                    .map(|v| v.visit_node(&node))
                    .unwrap_or_else(|| node.clone())
            }
            SyntaxKind::ElementAccessExpression => {
                if node.expression().unwrap().kind == SyntaxKind::SuperKeyword {
                    let argument_expression = node.as_element_access_expression()
                        .argument_expression
                        .clone();
                    return Some(self.create_super_element_access_in_async_method(
                        &argument_expression,
                    ));
                }
                self.super_access_visitor
                    .as_mut()
                    .map(|v| v.visit_node(&node))
                    .unwrap_or_else(|| node.clone())
            }
            SyntaxKind::FunctionExpression
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::Constructor
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::ClassExpression => node,
            _ => self
                .super_access_visitor
                .as_mut()
                .map(|v| v.visit_node(&node))
                .unwrap_or_else(|| node.clone()),
        }
        .into()
    }

    fn substitute_call_expression_with_super_access(&mut self, call_node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("substitute_call_expression_with_super_access"); 
        let call = call_node.as_call_expression();
        let expression = call.expression.clone();
        let target: Arc<Node>;

        if is_property_access_expression(&expression) {
            let super_binding = self.super_access_state.super_binding.clone().unwrap();
            target = self.transformer.factory().new_property_access_expression(
                &super_binding,
                None,
                expression.name().unwrap(),
                NodeFlags::empty(),
            );
        } else if is_element_access_expression(&expression) {
            let argument_expression = expression
                .as_element_access_expression()
                .argument_expression
                .clone();
            target = self.create_super_element_access_in_async_method(&argument_expression);
        } else {
            return match self.super_access_visitor.as_mut() {
                Some(v) => v.visit_node(call_node),
                None => call_node.clone(),
            };
        }

        let call_target = self.transformer.factory().new_property_access_expression(
            &target,
            None,
            &self.transformer.factory().new_identifier("call"),
            NodeFlags::empty(),
        );

        let mut all_args: Vec<Arc<Node>> = vec![self.transformer.factory().new_this_expression()];
        let visited_args = self
            .super_access_visitor
            .as_mut()
            .map(|v| v.visit_node_list(Some(call.arguments.as_ref())))
            .unwrap_or_default();
        all_args.extend(visited_args);

        let mut result = self.transformer.factory().new_call_expression(
            &call_target,
            None,
            None,
            self.transformer.factory().new_node_list(all_args),
            NodeFlags::empty(),
        );
        r39k13_defs::set_loc(&mut result, call_node.loc);
        result
    }

    fn create_super_element_access_in_async_method(
        &mut self,
        argument_expression: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("create_super_element_access_in_async_method"); 
        let super_index_binding = self.super_access_state.super_index_binding.clone().unwrap();
        let super_index_call = self.transformer.factory().new_call_expression(
            &super_index_binding,
            None,
            None,
            self.transformer
                .factory()
                .new_node_list(vec![argument_expression.clone()]),
            NodeFlags::empty(),
        );
        if self.super_access_state.has_super_property_assignment {
            return self.transformer.factory().new_property_access_expression(
                &super_index_call,
                None,
                &self.transformer.factory().new_identifier("value"),
                NodeFlags::empty(),
            );
        }
        super_index_call
    }

    pub fn create_super_access_variable_statement(&mut self) -> Arc<Node> { ::tsox_core::fntrace::enter("create_super_access_variable_statement"); 
        let f = self.transformer.factory();
        let mut accessors: Vec<Arc<Node>> = Vec::new();

        let captured: Vec<String> = self
            .super_access_state
            .captured_super_properties
            .as_ref()
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default();
        for name in &captured {
            let mut descriptor_properties: Vec<Arc<Node>> = Vec::new();

            let getter_body = f.new_property_access_expression(
                &f.new_keyword_expression(SyntaxKind::SuperKeyword),
                None,
                &f.new_identifier(name),
                NodeFlags::empty(),
            );
            let getter_parameters = f.new_node_list(Vec::new());
            let getter_arrow = f.new_arrow_function(
                None,
                None,
                &getter_parameters,
                None,
                None,
                &f.new_token(SyntaxKind::EqualsGreaterThanToken),
                &getter_body,
            );
            let getter = f.new_property_assignment(
                None,
                &f.new_identifier("get"),
                None,
                None,
                &getter_arrow,
            );
            descriptor_properties.push(getter);

            if self.super_access_state.has_super_property_assignment {
                let v_param = f.new_parameter_declaration(
                    None,
                    None,
                    &f.new_identifier("v"),
                    None,
                    None,
                    None,
                );
                let super_prop = f.new_property_access_expression(
                    &f.new_keyword_expression(SyntaxKind::SuperKeyword),
                    None,
                    &f.new_identifier(name),
                    NodeFlags::empty(),
                );
                let assign_expr = f.new_assignment_expression(&super_prop, &f.new_identifier("v"));
                let setter_parameters = f.new_node_list(vec![v_param]);
                let setter_arrow = f.new_arrow_function(
                    None,
                    None,
                    &setter_parameters,
                    None,
                    None,
                    &f.new_token(SyntaxKind::EqualsGreaterThanToken),
                    &assign_expr,
                );
                let setter = f.new_property_assignment(
                    None,
                    &f.new_identifier("set"),
                    None,
                    None,
                    &setter_arrow,
                );
                descriptor_properties.push(setter);
            }

            let descriptor =
                f.new_object_literal_expression(&f.new_node_list(descriptor_properties), false);
            let accessor =
                f.new_property_assignment(None, &f.new_identifier(name), None, None, &descriptor);
            accessors.push(accessor);
        }

        let descriptors_object = f.new_object_literal_expression(&f.new_node_list(accessors), true);

        let object_create_call = f.new_call_expression(
            &f.new_property_access_expression(
                &f.new_identifier("Object"),
                None,
                &f.new_identifier("create"),
                NodeFlags::empty(),
            ),
            None,
            None,
            f.new_node_list(vec![
                f.new_keyword_expression(SyntaxKind::NullKeyword),
                descriptors_object,
            ]),
            NodeFlags::empty(),
        );

        let super_binding = self.super_access_state.super_binding.clone().unwrap();
        let decl = f.new_variable_declaration(&super_binding, None, None, Some(&object_create_call));
        let list = f.new_node_list(vec![decl]);
        let decl_list = f.new_variable_declaration_list(&list, NodeFlags::Const);
        f.new_variable_statement(None, &decl_list)
    }
}

pub fn unwrap_innermost_statement_of_label(node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("unwrap_innermost_statement_of_label"); 
    let mut node = node.clone();
    loop {
        let statement = match &node.data {
            NodeData::LabeledStatement(d) => d.statement.clone(),
            _ => return node,
        };
        if statement.kind != SyntaxKind::LabeledStatement {
            return statement;
        }
        node = statement;
    }
}
