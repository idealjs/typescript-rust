use std::sync::Arc;
use tsox_core::collections::ordered_set::OrderedSet;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_frontend::ast::{Node, NodeFlags, SourceFile, SyntaxKind};
use tsox_frontend::ast::visitor::NodeVisitor;
use crate::printer::{AutoGenerateOptions, EmitContext, NodeFactory};
use tsox_checker::checker::mig::m2d::EmitResolver;
use tsox_frontend::ast::mig::m3g_2::is_super_property;
use tsox_frontend::ast::{is_assignment_operator, is_element_access_expression, is_property_access_expression};
use crate::mig::m4e_3::{assignment_target_contains_super_property, is_update_expression};
use crate::mig::m3m::TransformOptions;
use crate::mig::m4m::r36k5_defs::NodeDataExt;

#[path = "r36k17_defs.rs"]
pub mod r36k17_defs;
pub use r36k17_defs::*;

pub fn debug_fail(reason: &str) { ::tsox_core::fntrace::enter("debug_fail"); 
    let reason = if reason.is_empty() {
        "Debug failure.".to_string()
    } else {
        format!("Debug failure. {reason}")
    };
    panic!("{}", reason);
}

pub struct SuperAccessState {
    pub emit_context: EmitContext,
    pub captured_super_properties: Option<OrderedSet<String>>,
    pub has_super_element_access: bool,
    pub has_super_property_assignment: bool,
    pub super_binding: Arc<Node>,
    pub super_index_binding: Arc<Node>,
    pub super_access_visitor: NodeVisitor,
}

impl SuperAccessState {
    fn factory(&self) -> NodeFactory<'_> { ::tsox_core::fntrace::enter("factory"); 
        NodeFactory::new(&self.emit_context)
    }

    pub fn init_super_access_visitor(&mut self, emit_context: &EmitContext, _factory: NodeFactory<'_>) { ::tsox_core::fntrace::enter("init_super_access_visitor"); 
        self.super_access_visitor = emit_context.new_node_visitor(Self::visit_super_access_node);
    }

    fn visit_super_access_node(&mut self, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_super_access_node"); 
        match node.kind {
            SyntaxKind::CallExpression => {
                if is_super_property(&node.as_call_expression().expression) {
                    return Some(self.substitute_call_expression_with_super_access(&node));
                }
                Some(self.super_access_visitor.visit_node(&node))
            }
            SyntaxKind::PropertyAccessExpression => {
                if node.expression().unwrap().kind == SyntaxKind::SuperKeyword {
                    return Some(self.factory().new_property_access_expression(
                        &self.super_binding,
                        None,
                        node.name().unwrap(),
                        NodeFlags::empty(),
                    ));
                }
                Some(self.super_access_visitor.visit_node(&node))
            }
            SyntaxKind::ElementAccessExpression => {
                if node.expression().unwrap().kind == SyntaxKind::SuperKeyword {
                    return Some(self.create_super_element_access_in_async_method(
                        &node.as_element_access_expression().argument_expression,
                    ));
                }
                Some(self.super_access_visitor.visit_node(&node))
            }
            SyntaxKind::FunctionExpression
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::Constructor
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::ClassExpression => Some(node),
            _ => Some(self.super_access_visitor.visit_node(&node)),
        }
    }

    pub fn substitute_super_accesses_in_body(&mut self, body: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("substitute_super_accesses_in_body"); 
        self.super_access_visitor.visit_node(body)
    }

    fn substitute_call_expression_with_super_access(
        &mut self,
        call_node: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("substitute_call_expression_with_super_access"); 
        let call = call_node.as_call_expression();
        let expression = &call.expression;
        let target: Arc<Node>;

        if is_property_access_expression(expression) {
            target = self.factory().new_property_access_expression(
                &self.super_binding,
                None,
                &expression.as_property_access_expression().name,
                NodeFlags::empty(),
            );
        } else if is_element_access_expression(expression) {
            target = self.create_super_element_access_in_async_method(
                &expression.as_element_access_expression().argument_expression,
            );
        } else {
            return self.super_access_visitor.visit_node(call_node);
        }

        let call_target = self.factory().new_property_access_expression(
            &target,
            None,
            &self.factory().new_identifier("call"),
            NodeFlags::empty(),
        );

        let mut all_args: Vec<Arc<Node>> = vec![self.factory().new_this_expression()];
        let visited_args = self
            .super_access_visitor
            .visit_node_list(Some(call.arguments.as_ref()));
        all_args.extend(visited_args);

        let mut result = self.factory().new_call_expression(
            &call_target,
            None,
            None,
            self.factory().new_node_list(all_args),
            NodeFlags::empty(),
        );
        if let Some(result_data) = Arc::get_mut(&mut result) {
            result_data.loc = call_node.loc;
        }
        result
    }

    fn create_super_element_access_in_async_method(
        &mut self,
        argument_expression: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("create_super_element_access_in_async_method"); 
        let super_index_call = self.factory().new_call_expression(
            &self.super_index_binding,
            None,
            None,
            self.factory().new_node_list(vec![argument_expression.clone()]),
            NodeFlags::empty(),
        );
        if self.has_super_property_assignment {
            return self.factory().new_property_access_expression(
                &super_index_call,
                None,
                &self.factory().new_identifier("value"),
                NodeFlags::empty(),
            );
        }
        super_index_call
    }

    pub fn create_super_access_variable_statement(&mut self) -> Arc<Node> { ::tsox_core::fntrace::enter("create_super_access_variable_statement"); 
        let f = self.factory();
        let mut accessors: Vec<Arc<Node>> = Vec::new();

        for name in self.captured_super_properties.as_ref().unwrap().values() {
            let mut descriptor_properties: Vec<Arc<Node>> = Vec::new();

            let getter_body = f.new_property_access_expression(
                &f.new_keyword_expression(SyntaxKind::SuperKeyword),
                None,
                &f.new_identifier(name),
                NodeFlags::empty(),
            );
            let getter_arrow = f.new_arrow_function(
                None,
                None,
                &f.new_node_list(Vec::new()),
                None,
                None,
                &f.new_token(SyntaxKind::EqualsGreaterThanToken),
                &getter_body,
            );
            let getter = f.new_property_assignment(None, &f.new_identifier("get"), None, None, &getter_arrow);
            descriptor_properties.push(getter);

            if self.has_super_property_assignment {
                let v_param = f.new_parameter_declaration(None, None, &f.new_identifier("v"), None, None, None);
                let super_prop = f.new_property_access_expression(
                    &f.new_keyword_expression(SyntaxKind::SuperKeyword),
                    None,
                    &f.new_identifier(name),
                    NodeFlags::empty(),
                );
                let assign_expr = f.new_assignment_expression(&super_prop, &f.new_identifier("v"));
                let setter_arrow = f.new_arrow_function(
                    None,
                    None,
                    &f.new_node_list(vec![v_param]),
                    None,
                    None,
                    &f.new_token(SyntaxKind::EqualsGreaterThanToken),
                    &assign_expr,
                );
                let setter = f.new_property_assignment(None, &f.new_identifier("set"), None, None, &setter_arrow);
                descriptor_properties.push(setter);
            }

            let descriptor = f.new_object_literal_expression(&f.new_node_list(descriptor_properties), false);
            let accessor = f.new_property_assignment(None, &f.new_identifier(name), None, None, &descriptor);
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

        let decl = f.new_variable_declaration(&self.super_binding, None, None, Some(&object_create_call));
        let decl_list = f.new_variable_declaration_list(&f.new_node_list(vec![decl]), NodeFlags::Const);
        f.new_variable_statement(None, &decl_list)
    }

    pub fn track_super_access(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("track_super_access"); 
        if self.captured_super_properties.is_none() {
            return;
        }
        match node.kind {
            SyntaxKind::PropertyAccessExpression => {
                if node.expression().unwrap().kind == SyntaxKind::SuperKeyword {
                    self.captured_super_properties
                        .as_mut()
                        .unwrap()
                        .add(node.name().unwrap().text().to_string());
                }
            }
            SyntaxKind::ElementAccessExpression => {
                if node.expression().unwrap().kind == SyntaxKind::SuperKeyword {
                    self.has_super_element_access = true;
                }
            }
            SyntaxKind::BinaryExpression => {
                let binary = node.as_binary_expression();
                if is_assignment_operator(binary.operator_token.kind)
                    && assignment_target_contains_super_property(&binary.left)
                {
                    self.has_super_property_assignment = true;
                }
            }
            SyntaxKind::PrefixUnaryExpression => {
                if is_update_expression(node)
                    && assignment_target_contains_super_property(
                        &node.as_prefix_unary_expression().operand,
                    )
                {
                    self.has_super_property_assignment = true;
                }
            }
            SyntaxKind::PostfixUnaryExpression => {
                if is_update_expression(node)
                    && assignment_target_contains_super_property(
                        &node.as_postfix_unary_expression().operand,
                    )
                {
                    self.has_super_property_assignment = true;
                }
            }
            _ => {}
        }
    }
}

pub fn create_accessor_property_backing_field(
    f: &NodeFactory<'_>,
    node: &Arc<Node>,
    modifiers: Option<Arc<tsox_frontend::ast::ModifierList>>,
    initializer: Option<Arc<Node>>,
) -> Arc<Node> { ::tsox_core::fntrace::enter("create_accessor_property_backing_field"); 
    let data = node.as_property_declaration();
    let generated_name = f.new_generated_private_name_for_node_ex(
        &data.name,
        AutoGenerateOptions {
            suffix: "_accessor_storage".to_string(),
            ..Default::default()
        },
    );
    f.update_property_declaration(
        node,
        modifiers,
        &f.generated_name_node(&generated_name),
        None,
        None,
        initializer,
    )
}

pub struct ConstEnumInliningTransformer<'a> {
    pub(crate) emit_context: &'a EmitContext,
    pub(crate) compiler_options: &'a CompilerOptions,
    current_source_file: Option<Arc<SourceFile>>,
    pub(crate) emit_resolver: &'a EmitResolver,
}

pub fn new_const_enum_inlining_transformer<'a>(opts: &'a TransformOptions<'a>) -> ConstEnumInliningTransformer<'a> { ::tsox_core::fntrace::enter("new_const_enum_inlining_transformer"); 
    let compiler_options = opts.compiler_options;
    if compiler_options.get_isolated_modules() {
        debug_fail("const enums are not inlined under isolated modules");
    }
    ConstEnumInliningTransformer {
        emit_context: opts.context,
        compiler_options,
        current_source_file: None,
        emit_resolver: &opts.emit_resolver,
    }
}
