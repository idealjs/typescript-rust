#![allow(unused_imports, dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::visitor::NodeVisitor;

use crate::mig::m4f::AsyncTransformer;
use crate::mig::m4f_2::r38k5_defs::R38K5NodeVisitorExt;
use crate::mig::m4k_2::Transformer;

pub trait R39K19NodeVisitorExt {
    fn visit_embedded_statement(&mut self, node: &Arc<Node>) -> Option<Arc<Node>>;
}

impl R39K19NodeVisitorExt for NodeVisitor {
    fn visit_embedded_statement(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_embedded_statement"); 
        self.visit_node_opt(Some(node))
    }
}

impl AsyncTransformer {
    pub fn init_super_access_visitor(&mut self) { ::tsox_core::fntrace::enter("init_super_access_visitor"); 
        self.super_access_visitor = Some(
            self.emit_context()
                .new_node_visitor(|tx: &mut AsyncTransformer, node: Arc<Node>| tx.visit_super_access_node(&node)),
        );
    }

    pub fn visit_super_access_node(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_super_access_node"); 
        let mut visitor = self.super_access_visitor.take().unwrap();
        let result = self.visit_super_access_node_with(node, &mut visitor);
        self.super_access_visitor = Some(visitor);
        result
    }

    fn visit_super_access_node_with(
        &mut self,
        node: &Arc<Node>,
        visitor: &mut NodeVisitor,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_super_access_node_with"); 
        match node.kind {
            SyntaxKind::CallExpression => {
                let expression = node.expression().unwrap();
                if r39k19_is_super_property(&expression) {
                    return self.substitute_call_expression_with_super_access(node, visitor);
                }
                visitor.visit_each_child(node)
            }
            SyntaxKind::PropertyAccessExpression => {
                if node.expression().unwrap().kind == SyntaxKind::SuperKeyword {
                    let f = self.factory();
                    let super_binding = self.super_access.super_binding.clone().unwrap();
                    let name = node.name().unwrap();
                    return Some(f.new_property_access_expression(
                        &super_binding,
                        None,
                        &name,
                        NodeFlags::empty(),
                    ));
                }
                visitor.visit_each_child(node)
            }
            SyntaxKind::ElementAccessExpression => {
                if node.expression().unwrap().kind == SyntaxKind::SuperKeyword {
                    let argument_expression = match &node.data {
                        NodeData::ElementAccessExpression(d) => {
                            Arc::clone(&d.argument_expression)
                        }
                        _ => return visitor.visit_each_child(node),
                    };
                    return Some(
                        self.create_super_element_access_in_async_method(&argument_expression),
                    );
                }
                visitor.visit_each_child(node)
            }
            SyntaxKind::FunctionExpression
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::Constructor
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::ClassExpression => Some(Arc::clone(node)),
            _ => visitor.visit_each_child(node),
        }
    }

    fn substitute_call_expression_with_super_access(
        &mut self,
        call: &Arc<Node>,
        visitor: &mut NodeVisitor,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("substitute_call_expression_with_super_access"); 
        let expression = call.expression().unwrap();
        let target = if expression.kind == SyntaxKind::PropertyAccessExpression {
            let f = self.factory();
            let super_binding = self.super_access.super_binding.clone().unwrap();
            let name = expression.name().unwrap();
            Some(f.new_property_access_expression(
                &super_binding,
                None,
                &name,
                NodeFlags::empty(),
            ))
        } else if expression.kind == SyntaxKind::ElementAccessExpression {
            let argument_expression = match &expression.data {
                NodeData::ElementAccessExpression(d) => Arc::clone(&d.argument_expression),
                _ => return visitor.visit_each_child(call),
            };
            Some(self.create_super_element_access_in_async_method(&argument_expression))
        } else {
            return visitor.visit_each_child(call);
        };

        let f = self.factory();
        let call_target = f.new_property_access_expression(
            target.as_ref().unwrap(),
            None,
            &f.new_identifier("call"),
            NodeFlags::empty(),
        );

        let mut all_args = vec![f.new_this_expression()];
        if let NodeData::CallExpression(d) = &call.data {
            let visited_args = visitor.visit_node_list(Some(d.arguments.as_ref()));
            all_args.extend(visited_args);
        }

        let result = f.new_call_expression(
            &call_target,
            None,
            None,
            f.new_node_list(all_args),
            NodeFlags::empty(),
        );
        let mut result = result;
        if let Some(result_mut) = Arc::get_mut(&mut result) {
            result_mut.loc = call.loc;
        }
        Some(result)
    }

    fn create_super_element_access_in_async_method(&mut self, argument_expression: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("create_super_element_access_in_async_method"); 
        let super_index_binding = self.super_access.super_index_binding.clone().unwrap();
        let has_super_property_assignment = self.super_access.has_super_property_assignment;
        let f = self.factory();
        let super_index_call = f.new_call_expression(
            &super_index_binding,
            None,
            None,
            f.new_node_list(vec![Arc::clone(argument_expression)]),
            NodeFlags::empty(),
        );
        if has_super_property_assignment {
            return f.new_property_access_expression(
                &super_index_call,
                None,
                &f.new_identifier("value"),
                NodeFlags::empty(),
            );
        }
        super_index_call
    }
}

fn r39k19_is_super_property(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("r39k19_is_super_property"); 
    matches!(
        node.kind,
        SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression
    ) && node
        .expression()
        .map(|e| e.kind == SyntaxKind::SuperKeyword)
        .unwrap_or(false)
}

pub fn async_transformer_visit_entry(_tx: &mut Transformer, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("async_transformer_visit_entry"); 
    Some(node)
}

pub fn class_fields_transformer_visit_entry(
    _tx: &mut Transformer,
    node: Arc<Node>,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("class_fields_transformer_visit_entry"); 
    Some(node)
}
