#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::node_source_file::SourceFile;
use tsox_frontend::ast::{Node, NodeList};
use tsox_frontend::format::mig::m4o_2::{CommentState, SourceMapState};

use crate::mig::m4q::r39k08_defs::EmitContextExtK08;

use super::Printer;

pub trait R39k06NodeAccessors {
    fn k06_asserts_modifier(&self) -> Option<&Node>;
    fn k06_parameter_name(&self) -> &Node;
    fn k06_type_name(&self) -> &Node;
    fn k06_expr_name(&self) -> &Node;
    fn k06_types(&self) -> &NodeList;
    fn k06_operator(&self) -> tsox_frontend::ast::SyntaxKind;
    fn k06_constraint(&self) -> Option<&Node>;
    fn k06_default_type(&self) -> Option<&Node>;
    fn k06_property_list(&self) -> &NodeList;
    fn k06_element_list(&self) -> &NodeList;
    fn k06_then_statement(&self) -> &Node;
    fn k06_else_statement(&self) -> Option<&Node>;
    fn k06_case_block(&self) -> &Node;
    fn k06_clauses(&self) -> &NodeList;
    fn k06_variable_declaration(&self) -> Option<&Node>;
    fn k06_block(&self) -> &Node;
    fn k06_import_clause(&self) -> Option<&Node>;
    fn k06_named_bindings(&self) -> Option<&Node>;
}

impl R39k06NodeAccessors for Node {
    fn k06_asserts_modifier(&self) -> Option<&Node> { ::tsox_core::fntrace::enter("k06_asserts_modifier"); 
        match &self.data {
            NodeData::TypePredicateNode(d) => d.asserts_modifier.as_deref(),
            _ => None,
        }
    }

    fn k06_parameter_name(&self) -> &Node { ::tsox_core::fntrace::enter("k06_parameter_name"); 
        match &self.data {
            NodeData::TypePredicateNode(d) => &d.parameter_name,
            _ => panic!("parameter_name on {:?}", self.kind),
        }
    }

    fn k06_type_name(&self) -> &Node { ::tsox_core::fntrace::enter("k06_type_name"); 
        match &self.data {
            NodeData::TypeReferenceNode(d) => &d.type_name,
            _ => panic!("type_name on {:?}", self.kind),
        }
    }

    fn k06_expr_name(&self) -> &Node { ::tsox_core::fntrace::enter("k06_expr_name"); 
        match &self.data {
            NodeData::TypeQueryNode(d) => &d.expr_name,
            _ => panic!("expr_name on {:?}", self.kind),
        }
    }

    fn k06_types(&self) -> &NodeList { ::tsox_core::fntrace::enter("k06_types"); 
        match &self.data {
            NodeData::UnionTypeNode(d) => &d.types,
            NodeData::IntersectionTypeNode(d) => &d.types,
            _ => panic!("types on {:?}", self.kind),
        }
    }

    fn k06_operator(&self) -> tsox_frontend::ast::SyntaxKind { ::tsox_core::fntrace::enter("k06_operator"); 
        match &self.data {
            NodeData::TypeOperatorNode(d) => d.operator,
            _ => panic!("operator on {:?}", self.kind),
        }
    }

    fn k06_constraint(&self) -> Option<&Node> { ::tsox_core::fntrace::enter("k06_constraint"); 
        match &self.data {
            NodeData::TypeParameterDeclaration(d) => d.constraint.as_deref(),
            _ => None,
        }
    }

    fn k06_default_type(&self) -> Option<&Node> { ::tsox_core::fntrace::enter("k06_default_type"); 
        match &self.data {
            NodeData::TypeParameterDeclaration(d) => d.default_type.as_deref(),
            _ => None,
        }
    }

    fn k06_property_list(&self) -> &NodeList { ::tsox_core::fntrace::enter("k06_property_list"); 
        match &self.data {
            NodeData::ObjectLiteralExpression(d) => &d.properties,
            _ => panic!("property_list on {:?}", self.kind),
        }
    }

    fn k06_element_list(&self) -> &NodeList { ::tsox_core::fntrace::enter("k06_element_list"); 
        match &self.data {
            NodeData::ArrayLiteralExpression(d) => &d.elements,
            _ => panic!("element_list on {:?}", self.kind),
        }
    }

    fn k06_then_statement(&self) -> &Node { ::tsox_core::fntrace::enter("k06_then_statement"); 
        match &self.data {
            NodeData::IfStatement(d) => &d.then_statement,
            _ => panic!("then_statement on {:?}", self.kind),
        }
    }

    fn k06_else_statement(&self) -> Option<&Node> { ::tsox_core::fntrace::enter("k06_else_statement"); 
        match &self.data {
            NodeData::IfStatement(d) => d.else_statement.as_deref(),
            _ => None,
        }
    }

    fn k06_case_block(&self) -> &Node { ::tsox_core::fntrace::enter("k06_case_block"); 
        match &self.data {
            NodeData::SwitchStatement(d) => &d.case_block,
            _ => panic!("case_block on {:?}", self.kind),
        }
    }

    fn k06_clauses(&self) -> &NodeList { ::tsox_core::fntrace::enter("k06_clauses"); 
        match &self.data {
            NodeData::CaseBlock(d) => &d.clauses,
            _ => panic!("clauses on {:?}", self.kind),
        }
    }

    fn k06_variable_declaration(&self) -> Option<&Node> { ::tsox_core::fntrace::enter("k06_variable_declaration"); 
        match &self.data {
            NodeData::CatchClause(d) => d.variable_declaration.as_deref(),
            _ => None,
        }
    }

    fn k06_block(&self) -> &Node { ::tsox_core::fntrace::enter("k06_block"); 
        match &self.data {
            NodeData::CatchClause(d) => &d.block,
            _ => panic!("block on {:?}", self.kind),
        }
    }

    fn k06_import_clause(&self) -> Option<&Node> { ::tsox_core::fntrace::enter("k06_import_clause"); 
        match &self.data {
            NodeData::ImportDeclaration(d) => d.import_clause.as_deref(),
            _ => None,
        }
    }

    fn k06_named_bindings(&self) -> Option<&Node> { ::tsox_core::fntrace::enter("k06_named_bindings"); 
        match &self.data {
            NodeData::ImportClause(d) => d.named_bindings.as_deref(),
            _ => None,
        }
    }
}

pub trait R39k06EmitContextExt {
    fn k06_emit_flags(&self, node: &Node) -> u32;
    fn k06_get_synthetic_trailing_comments(
        &self,
        node: &Node,
    ) -> Vec<crate::printer::mig::m4m_2::SynthesizedComment>;
    fn k06_factory(&self) -> tsox_frontend::format::mig::m4o::NodeFactory;
}

impl R39k06EmitContextExt for tsox_frontend::format::mig::m4o_2::EmitContext {
    fn k06_emit_flags(&self, node: &Node) -> u32 { ::tsox_core::fntrace::enter("k06_emit_flags"); 
        self.emit_flags_of(node)
    }

    fn k06_get_synthetic_trailing_comments(
        &self,
        _node: &Node,
    ) -> Vec<crate::printer::mig::m4m_2::SynthesizedComment> { ::tsox_core::fntrace::enter("k06_get_synthetic_trailing_comments"); 
        Vec::new()
    }

    fn k06_factory(&self) -> tsox_frontend::format::mig::m4o::NodeFactory { ::tsox_core::fntrace::enter("k06_factory"); 
        tsox_frontend::format::mig::m4o::new_node_factory(
            tsox_frontend::format::mig::m4o::EmitContext::default(),
        )
    }
}

pub trait R39k06NodeFactoryExt {
    fn update_property_access_expression(
        &self,
        node: &Node,
        expression: Option<Arc<Node>>,
        question_dot_token: Option<&Arc<Node>>,
        name: &Arc<Node>,
    ) -> Arc<Node>;
    fn update_element_access_expression(
        &self,
        node: &Node,
        expression: Option<Arc<Node>>,
        question_dot_token: Option<&Arc<Node>>,
        argument_expression: &Arc<Node>,
    ) -> Arc<Node>;
    fn update_call_expression(
        &self,
        node: &Node,
        expression: Option<Arc<Node>>,
        question_dot_token: Option<&Arc<Node>>,
        type_arguments: Option<&Arc<NodeList>>,
        arguments: &Arc<NodeList>,
    ) -> Arc<Node>;
    fn update_tagged_template_expression(
        &self,
        node: &Node,
        tag: Option<Arc<Node>>,
        question_dot_token: Option<&Arc<Node>>,
        type_arguments: Option<&Arc<NodeList>>,
        template: &Arc<Node>,
    ) -> Arc<Node>;
}

impl R39k06NodeFactoryExt for tsox_frontend::format::mig::m4o::NodeFactory {
    fn update_property_access_expression(
        &self,
        node: &Node,
        expression: Option<Arc<Node>>,
        question_dot_token: Option<&Arc<Node>>,
        name: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_property_access_expression"); 
        let old = match &node.data {
            tsox_frontend::ast::node_data_generated::NodeData::PropertyAccessExpression(d) => d,
            _ => panic!("property_access_expression expected: {:?}", node.kind),
        };
        let expression = expression.unwrap_or_else(|| Arc::clone(&old.expression));
        let question_dot_token = question_dot_token
            .cloned()
            .or_else(|| old.question_dot_token.clone());
        let name = Arc::clone(name);
        let mut updated = tsox_frontend::ast::node_node::Node::with_loc(
            node.kind,
            tsox_frontend::ast::node_data_generated::NodeData::PropertyAccessExpression(
                tsox_frontend::ast::node_data_generated::PropertyAccessExpressionData {
                    expression,
                    question_dot_token,
                    name,
                },
            ),
            node.loc,
        );
        updated.flags = node.flags;
        Arc::new(updated)
    }

    fn update_element_access_expression(
        &self,
        node: &Node,
        expression: Option<Arc<Node>>,
        question_dot_token: Option<&Arc<Node>>,
        argument_expression: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_element_access_expression"); 
        let old = match &node.data {
            tsox_frontend::ast::node_data_generated::NodeData::ElementAccessExpression(d) => d,
            _ => panic!("element_access_expression expected: {:?}", node.kind),
        };
        let expression = expression.unwrap_or_else(|| Arc::clone(&old.expression));
        let question_dot_token = question_dot_token
            .cloned()
            .or_else(|| old.question_dot_token.clone());
        let argument_expression = Arc::clone(argument_expression);
        let mut updated = tsox_frontend::ast::node_node::Node::with_loc(
            node.kind,
            tsox_frontend::ast::node_data_generated::NodeData::ElementAccessExpression(
                tsox_frontend::ast::node_data_generated::ElementAccessExpressionData {
                    expression,
                    question_dot_token,
                    argument_expression,
                },
            ),
            node.loc,
        );
        updated.flags = node.flags;
        Arc::new(updated)
    }

    fn update_call_expression(
        &self,
        node: &Node,
        expression: Option<Arc<Node>>,
        question_dot_token: Option<&Arc<Node>>,
        type_arguments: Option<&Arc<NodeList>>,
        arguments: &Arc<NodeList>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_call_expression"); 
        let old = match &node.data {
            tsox_frontend::ast::node_data_generated::NodeData::CallExpression(d) => d,
            _ => panic!("call_expression expected: {:?}", node.kind),
        };
        let expression = expression.unwrap_or_else(|| Arc::clone(&old.expression));
        let question_dot_token = question_dot_token
            .cloned()
            .or_else(|| old.question_dot_token.clone());
        let type_arguments = type_arguments.map(Arc::clone).or_else(|| old.type_arguments.clone());
        let arguments = Arc::clone(arguments);
        let mut updated = tsox_frontend::ast::node_node::Node::with_loc(
            node.kind,
            tsox_frontend::ast::node_data_generated::NodeData::CallExpression(
                tsox_frontend::ast::node_data_generated::CallExpressionData {
                    expression,
                    question_dot_token,
                    type_arguments,
                    arguments,
                },
            ),
            node.loc,
        );
        updated.flags = node.flags;
        Arc::new(updated)
    }

    fn update_tagged_template_expression(
        &self,
        node: &Node,
        tag: Option<Arc<Node>>,
        question_dot_token: Option<&Arc<Node>>,
        type_arguments: Option<&Arc<NodeList>>,
        template: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_tagged_template_expression"); 
        let old = match &node.data {
            tsox_frontend::ast::node_data_generated::NodeData::TaggedTemplateExpression(d) => d,
            _ => panic!("tagged_template_expression expected: {:?}", node.kind),
        };
        let tag = tag.unwrap_or_else(|| Arc::clone(&old.tag));
        let question_dot_token = question_dot_token
            .cloned()
            .or_else(|| old.question_dot_token.clone());
        let type_arguments = type_arguments.map(Arc::clone).or_else(|| old.type_arguments.clone());
        let template = Arc::clone(template);
        let mut updated = tsox_frontend::ast::node_node::Node::with_loc(
            node.kind,
            tsox_frontend::ast::node_data_generated::NodeData::TaggedTemplateExpression(
                tsox_frontend::ast::node_data_generated::TaggedTemplateExpressionData {
                    tag,
                    question_dot_token,
                    type_arguments,
                    template,
                },
            ),
            node.loc,
        );
        updated.flags = node.flags;
        Arc::new(updated)
    }
}

#[derive(Default)]
pub struct PrinterState39k06 {
    pub comment_state: Option<CommentState>,
    pub source_map_state: Option<SourceMapState>,
}

impl Printer {
    pub(crate) fn k06_write_synthesized_comment(&mut self, comment: &crate::printer::mig::m4m_2::SynthesizedComment) { ::tsox_core::fntrace::enter("k06_write_synthesized_comment"); 
        let text = crate::mig::m4r_4::format_synthesized_comment(comment);
        self.write_as(&text, super::WriteKind::Comment);
    }
}
