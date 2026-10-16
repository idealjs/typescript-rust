#![allow(unused_imports)]
#[path = "r37k14_defs.rs"]
pub mod r37k14_defs;

use std::collections::HashSet;
use std::sync::Arc;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::{Node, NodeList, SourceFile, SyntaxKind};
use tsox_frontend::ast::subtree_facts::SubtreeFacts;
use tsox_frontend::ast::mig::m3c_2::subtree_facts;
use tsox_frontend::ast::is_binding_pattern;
use crate::printer::{EmitContext, NodeFactory};
use crate::mig::m3m::TransformOptions;
use crate::mig::m4e::r37k1_defs::R37K1DataExt;
use crate::mig::m4m_5::r38k9_defs::{
    R38K9ArrowFnDataExt, R38K9FnExprDataExt, R38K9FunctionDeclDataExt, R38K9MethodDeclDataExt,
    R38K9NodeCastExt, R38K9ParamDeclDataExt, R38K9SourceFileNodeExt,
};
use crate::mig::m4i::r39k11_defs::{R39K11FunctionDeclCastExt, R39K11ObjectRestExt};
use self::r37k14_defs::R37K14DataExt;
pub struct ObjectRestSpreadTransformer {
    pub(crate) emit_context: EmitContext,
    pub(crate) compiler_options: Arc<CompilerOptions>,
    pub(crate) in_exported_variable_statement: bool,
    pub(crate) expression_result_is_unused: bool,
    parameters_with_preceding_object_rest_or_spread: Option<HashSet<u64>>,
}

pub type OldParamScope = Option<HashSet<u64>>;

pub fn new_object_rest_spread_transformer(opts: &TransformOptions) -> ObjectRestSpreadTransformer { ::tsox_core::fntrace::enter("new_object_rest_spread_transformer"); 
    ObjectRestSpreadTransformer {
        emit_context: opts.context.clone(),
        compiler_options: Arc::new(opts.compiler_options.clone()),
        in_exported_variable_statement: false,
        expression_result_is_unused: false,
        parameters_with_preceding_object_rest_or_spread: None,
    }
}

impl ObjectRestSpreadTransformer {
    pub(crate) fn factory(&self) -> NodeFactory<'_> { ::tsox_core::fntrace::enter("factory"); 
        NodeFactory::new(&self.emit_context)
    }

}
impl ObjectRestSpreadTransformer {
    pub fn visit(&mut self, node: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit"); 
        if !subtree_facts(&node).intersects(SubtreeFacts::CONTAINS_ES_OBJECT_REST_OR_SPREAD)
            && self.parameters_with_preceding_object_rest_or_spread.is_none()
        {
            return node;
        }
        let expression_result_is_unused = self.expression_result_is_unused;
        self.expression_result_is_unused = false;
        let result = match node.kind {
            SyntaxKind::SourceFile => self.visit_source_file(node),
            SyntaxKind::ObjectLiteralExpression => self.visit_object_literal_expression(node),
            SyntaxKind::BinaryExpression => {
                self.visit_binary_expression(node, expression_result_is_unused)
            }
            SyntaxKind::ExpressionStatement => {
                self.expression_result_is_unused = true;
                self.visit_each_child(&node)
            }
            SyntaxKind::ParenthesizedExpression => {
                self.expression_result_is_unused = expression_result_is_unused;
                self.visit_each_child(&node)
            }
            SyntaxKind::ForOfStatement => self.visit_for_of_statement(node),
            SyntaxKind::VariableStatement => self.visit_variable_statement(node),
            SyntaxKind::VariableDeclaration => self.visit_variable_declaration(node),
            SyntaxKind::CatchClause => self.visit_catch_clause(node),
            SyntaxKind::Parameter => self.visit_parameter(node),
            SyntaxKind::Constructor => self.visit_constructor_declaration(node),
            SyntaxKind::GetAccessor => self.visit_get_accessor_declaration(node),
            SyntaxKind::SetAccessor => self.visit_set_accessor_declaration(node),
            SyntaxKind::MethodDeclaration => self.visit_method_declaration(node),
            SyntaxKind::FunctionDeclaration => self.visit_function_declaration(node),
            SyntaxKind::ArrowFunction => self.visit_arrow_function(node),
            SyntaxKind::FunctionExpression => self.visit_function_expression(node),
            _ => self.visit_each_child(&node),
        };
        self.expression_result_is_unused = expression_result_is_unused;
        result
    }

}
impl ObjectRestSpreadTransformer {
    fn visit_source_file(&mut self, node: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_source_file"); 
        let data = node.as_source_file_data();
        let visited_statements: Vec<Arc<Node>> = data
            .statements
            .nodes
            .iter()
            .map(|s| self.visit(s.clone()))
            .collect();
        let visited = self.factory().update_source_file(
            &node,
            Arc::new(NodeList {
                nodes: visited_statements,
                loc: data.statements.loc,
            }),
        );
        let helpers = self.emit_context.read_emit_helpers();
        self.emit_context.add_emit_helper(&visited, &helpers);
        visited
    }

}
impl ObjectRestSpreadTransformer {
    fn visit_parameter(&mut self, node: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_parameter"); 
        let data = node.as_parameter_declaration();
        if let Some(params) = &self.parameters_with_preceding_object_rest_or_spread {
            if params.contains(&node.id()) {
                let mut name = data.name().clone();
                if is_binding_pattern(&name) {
                    let generated = self.factory().new_generated_name_for_node(&node);
                    name = self.factory().generated_name_node(&generated);
                }
                return self.factory().update_parameter_declaration(
                    &node,
                    node.modifiers().cloned(),
                    data.dot_dot_dot_token(),
                    &name,
                    None,
                    None,
                    None,
                );
            }
        }
        if subtree_facts(&node).intersects(SubtreeFacts::CONTAINS_OBJECT_REST_OR_SPREAD) {
            let generated = self.factory().new_generated_name_for_node(&node);
            let name = self.factory().generated_name_node(&generated);
            let initializer = self.visit_node(data.initializer());
            return self.factory().update_parameter_declaration(
                &node,
                node.modifiers().cloned(),
                data.dot_dot_dot_token(),
                &name,
                None,
                None,
                initializer.as_ref(),
            );
        }
        self.visit_each_child(&node)
    }

}
impl ObjectRestSpreadTransformer {
    fn collect_parameters_with_preceding_object_rest_or_spread(&self, node: &Arc<Node>) -> Option<HashSet<u64>> { ::tsox_core::fntrace::enter("collect_parameters_with_preceding_object_rest_or_spread"); 
        let mut result: Option<HashSet<u64>> = None;
        if let Some(list) = node.parameters() {
            for parameter in &list.nodes {
                if let Some(set) = &mut result {
                    set.insert(parameter.id());
                } else if subtree_facts(parameter).intersects(SubtreeFacts::CONTAINS_OBJECT_REST_OR_SPREAD) {
                    result = Some(HashSet::new());
                }
            }
        }
        result
    }

}
impl ObjectRestSpreadTransformer {
    fn enter_parameter_list_context(&mut self, node: &Arc<Node>) -> OldParamScope { ::tsox_core::fntrace::enter("enter_parameter_list_context"); 
        let old = self.parameters_with_preceding_object_rest_or_spread.take();
        self.parameters_with_preceding_object_rest_or_spread =
            self.collect_parameters_with_preceding_object_rest_or_spread(node);
        old
    }

}
impl ObjectRestSpreadTransformer {
    fn exit_parameter_list_context(&mut self, scope: OldParamScope) { ::tsox_core::fntrace::enter("exit_parameter_list_context"); 
        self.parameters_with_preceding_object_rest_or_spread = scope;
    }

}
impl ObjectRestSpreadTransformer {
    fn visit_constructor_declaration(&mut self, node: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_constructor_declaration"); 
        let old = self.enter_parameter_list_context(&node);
        let body = self.transform_function_body(&node);
        let parameters = self.visit_nodes(node.parameters());
        let result = self.factory().update_constructor_declaration(
            &node,
            node.modifiers().cloned(),
            None,
            parameters.as_deref().expect("constructor parameters"),
            None,
            None,
            body.as_ref(),
        );
        self.exit_parameter_list_context(old);
        result
    }

}
impl ObjectRestSpreadTransformer {
    fn visit_get_accessor_declaration(&mut self, node: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_get_accessor_declaration"); 
        let old = self.enter_parameter_list_context(&node);
        let name = self
            .visit_node(node.name())
            .unwrap_or_else(|| node.name().unwrap().clone());
        let body = self.transform_function_body(&node);
        let parameters = self.visit_nodes(node.parameters());
        let result = self.factory().update_get_accessor_declaration(
            &node,
            node.modifiers().cloned(),
            &name,
            None,
            parameters.as_deref().expect("accessor parameters"),
            None,
            None,
            body.as_ref(),
        );
        self.exit_parameter_list_context(old);
        result
    }

}
impl ObjectRestSpreadTransformer {
    fn visit_set_accessor_declaration(&mut self, node: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_set_accessor_declaration"); 
        let old = self.enter_parameter_list_context(&node);
        let name = self
            .visit_node(node.name())
            .unwrap_or_else(|| node.name().unwrap().clone());
        let body = self.transform_function_body(&node);
        let parameters = self.visit_nodes(node.parameters());
        let result = self.factory().update_set_accessor_declaration(
            &node,
            node.modifiers().cloned(),
            &name,
            None,
            parameters.as_deref().expect("accessor parameters"),
            None,
            None,
            body.as_ref(),
        );
        self.exit_parameter_list_context(old);
        result
    }

}
impl ObjectRestSpreadTransformer {
    fn visit_method_declaration(&mut self, node: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_method_declaration"); 
        let old = self.enter_parameter_list_context(&node);
        let data = node.as_method_declaration();
        let name = self
            .visit_node(node.name())
            .unwrap_or_else(|| node.name().unwrap().clone());
        let body = self.transform_function_body(&node);
        let parameters = self.visit_nodes(node.parameters());
        let asterisk_token = data.asterisk_token().cloned();
        let postfix_token = data.postfix_token().cloned();
        let result = self.factory().update_method_declaration(
            &node,
            node.modifiers().cloned(),
            asterisk_token.as_ref(),
            &name,
            postfix_token,
            None,
            parameters.as_deref().expect("method parameters"),
            None,
            None,
            body.as_ref(),
        );
        self.exit_parameter_list_context(old);
        result
    }

}
impl ObjectRestSpreadTransformer {
    fn visit_function_declaration(&mut self, node: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_function_declaration"); 
        let old = self.enter_parameter_list_context(&node);
        let data = node.as_function_declaration();
        let name = self.visit_node(node.name());
        let body = self.transform_function_body(&node);
        let parameters = self.visit_nodes(node.parameters());
        let asterisk_token = data.asterisk_token().cloned();
        let result = self.factory().update_function_declaration(
            &node,
            node.modifiers().cloned(),
            asterisk_token.as_ref(),
            name.as_ref(),
            None,
            parameters.as_deref().expect("function parameters"),
            None,
            None,
            body.as_ref(),
        );
        self.exit_parameter_list_context(old);
        result
    }

}
impl ObjectRestSpreadTransformer {
    fn visit_arrow_function(&mut self, node: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_arrow_function"); 
        let old = self.enter_parameter_list_context(&node);
        let data = node.as_arrow_function();
        let body = self
            .transform_function_body(&node)
            .or_else(|| node.body().cloned())
            .expect("arrow function body");
        let parameters = self.visit_nodes(node.parameters());
        let result = self.factory().update_arrow_function(
            &node,
            node.modifiers().cloned(),
            None,
            parameters.as_deref().expect("arrow parameters"),
            None,
            None,
            &data.equals_greater_than_token().clone(),
            &body,
        );
        self.exit_parameter_list_context(old);
        result
    }

}
impl ObjectRestSpreadTransformer {
    fn visit_function_expression(&mut self, node: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_function_expression"); 
        let old = self.enter_parameter_list_context(&node);
        let data = node.as_function_expression();
        let name = self.visit_node(node.name());
        let body = self.transform_function_body(&node);
        let parameters = self.visit_nodes(node.parameters());
        let asterisk_token = data.asterisk_token().cloned();
        let result = self.factory().update_function_expression(
            &node,
            node.modifiers().cloned(),
            asterisk_token.as_ref(),
            name.as_ref(),
            None,
            parameters.as_deref().expect("function expression parameters"),
            None,
            None,
            body.as_ref(),
        );
        self.exit_parameter_list_context(old);
        result
    }

}
