#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::{Arc, OnceLock};

use tsox_frontend::ast::mig::m3g_3::try_get_property_name_of_binding_or_assignment_element;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated as ndg;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::node_data_generated::is_computed_property_name;
use tsox_frontend::ast::node_flags::{ModifierFlags, NodeFlags};
use tsox_frontend::ast::NodeList;
use tsox_frontend::ast::node::ModifierList;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_core::core::text::TextRange;
use tsox_frontend::format::mig::m4o::rest_helper;
use tsox_frontend::format::mig::m4o_2::EmitHelper;

use crate::mig::m4h_8::r36k26_defs::EmitContextCommentRangeR36k26;
use crate::mig::m4l_6::TypeEraserTransformer;
use crate::printer::{EmitContext, NodeFactory};

fn k3_rest_helper() -> &'static Arc<EmitHelper> { ::tsox_core::fntrace::enter("k3_rest_helper"); 
    static HELPER: OnceLock<Arc<EmitHelper>> = OnceLock::new();
    HELPER.get_or_init(|| Arc::new(rest_helper()))
}

impl<'a> NodeFactory<'a> {
    pub fn new_partially_emitted_expression(&self, expression: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_partially_emitted_expression"); 
        Arc::new(Node::new(
            SyntaxKind::PartiallyEmittedExpression,
            NodeData::PartiallyEmittedExpression(ndg::PartiallyEmittedExpressionData {
                expression: Arc::clone(expression),
            }),
        ))
    }

    pub fn update_property_declaration(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        name: &Arc<Node>,
        postfix_token: Option<Arc<Node>>,
        type_node: Option<Arc<Node>>,
        initializer: Option<Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_property_declaration"); 
        let mut updated = Node::new(
            SyntaxKind::PropertyDeclaration,
            NodeData::PropertyDeclaration(ndg::PropertyDeclarationData {
                modifiers,
                name: Arc::clone(name),
                postfix_token,
                type_node,
                initializer,
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_variable_declaration_node(
        &self,
        node: &Arc<Node>,
        name: &Arc<Node>,
        exclamation_token: Option<Arc<Node>>,
        type_node: Option<Arc<Node>>,
        initializer: Option<Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_variable_declaration_node"); 
        let mut updated = Node::new(
            SyntaxKind::VariableDeclaration,
            NodeData::VariableDeclaration(ndg::VariableDeclarationData {
                name: Arc::clone(name),
                exclamation_token,
                type_node,
                initializer,
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_heritage_clause(
        &self,
        node: &Arc<Node>,
        token: SyntaxKind,
        types: &NodeList,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_heritage_clause"); 
        let mut updated = Node::new(
            SyntaxKind::HeritageClause,
            NodeData::HeritageClause(ndg::HeritageClauseData {
                token,
                types: Arc::new(crate::mig::m4h_4::r36k9_defs::cloned_node_list(types)),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_new_expression(
        &self,
        node: &Arc<Node>,
        expression: &Arc<Node>,
        type_arguments: Option<Arc<NodeList>>,
        arguments: &NodeList,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_new_expression"); 
        let mut updated = Node::new(
            SyntaxKind::NewExpression,
            NodeData::NewExpression(ndg::NewExpressionData {
                expression: Arc::clone(expression),
                type_arguments,
                arguments: Some(Arc::new(
                    crate::mig::m4h_4::r36k9_defs::cloned_node_list(arguments),
                )),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_jsx_self_closing_element(
        &self,
        node: &Arc<Node>,
        tag_name: &Arc<Node>,
        type_arguments: Option<Arc<NodeList>>,
        attributes: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_jsx_self_closing_element"); 
        let mut updated = Node::new(
            SyntaxKind::JsxSelfClosingElement,
            NodeData::JsxSelfClosingElement(ndg::JsxSelfClosingElementData {
                tag_name: Arc::clone(tag_name),
                type_arguments,
                attributes: Arc::clone(attributes),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_jsx_opening_element(
        &self,
        node: &Arc<Node>,
        tag_name: &Arc<Node>,
        type_arguments: Option<Arc<NodeList>>,
        attributes: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_jsx_opening_element"); 
        let mut updated = Node::new(
            SyntaxKind::JsxOpeningElement,
            NodeData::JsxOpeningElement(ndg::JsxOpeningElementData {
                tag_name: Arc::clone(tag_name),
                type_arguments,
                attributes: Arc::clone(attributes),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn new_rest_helper(
        &self,
        value: &Arc<Node>,
        elements: &[Arc<Node>],
        computed_temp_variables: &[Arc<Node>],
        location: TextRange,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_rest_helper"); 
        self.emit_context
            .request_emit_helper(k3_rest_helper());
        let mut property_names: Vec<Arc<Node>> = Vec::new();
        let mut computed_temp_variable_offset = 0;
        for element in elements.iter().take(elements.len().saturating_sub(1)) {
            let Some(property_name) = try_get_property_name_of_binding_or_assignment_element(element)
            else {
                continue;
            };
            if is_computed_property_name(&property_name) {
                let temp = &computed_temp_variables[computed_temp_variable_offset];
                computed_temp_variable_offset += 1;
                property_names.push(self.new_conditional_expression(
                    &self.new_type_check(temp, "symbol"),
                    &self.new_token(SyntaxKind::QuestionToken),
                    temp,
                    &self.new_token(SyntaxKind::ColonToken),
                    &self.new_binary_expression(
                        None,
                        temp,
                        None,
                        &self.new_token(SyntaxKind::PlusToken),
                        &self.new_string_literal("", 0),
                    ),
                ));
            } else {
                property_names.push(self.new_string_literal_from_node(&property_name));
            }
        }
        let mut prop_names = self.new_array_literal_expression(
            &self.new_node_list(property_names),
            false,
        );
        if let Some(node) = Arc::get_mut(&mut prop_names) {
            node.loc = location;
        }
        self.new_call_expression(
            &self.new_unscoped_helper_name("__rest"),
            None,
            None,
            self.new_node_list(vec![Arc::clone(value), prop_names]),
            NodeFlags::empty(),
        )
    }
}

pub fn visit_accessor_declaration_k3(
    tx: &mut TypeEraserTransformer,
    node: &Arc<Node>,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_accessor_declaration_k3"); 
    let is_get = node.kind == SyntaxKind::GetAccessor;
    if tsox_frontend::ast::node_is_missing(node.body())
        && node.has_syntactic_modifier(ModifierFlags::Ambient)
    {
        return None;
    }
    let body = tx
        .visitor()
        .visit_node_opt(node.body())
        .unwrap_or_else(|| {
            tx.factory()
                .new_block(&tx.factory().new_node_list(Vec::new()), false)
        });
    let mut visited = tx.visitor();
    let modifiers = node.modifiers().cloned();
    let name = visited.visit_node(node.name().expect("accessor name"));
    let parameters = node
        .parameters()
        .map(|p| {
            let mut list = NodeList::new(visited.visit_node_list(Some(p.as_ref())));
            list.loc = p.loc;
            list
        })
        .unwrap_or_else(|| NodeList::new(Vec::new()));
    let factory = tx.factory();
    let body_ref = Some(&body);
    Some(if is_get {
        factory.update_get_accessor_declaration(
            node, modifiers, &name, None, &parameters, None, None, body_ref,
        )
    } else {
        factory.update_set_accessor_declaration(
            node, modifiers, &name, None, &parameters, None, None, body_ref,
        )
    })
}
