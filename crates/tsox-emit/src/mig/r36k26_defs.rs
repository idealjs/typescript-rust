#![allow(unused_imports, dead_code)]

use std::sync::Arc;
use std::sync::OnceLock;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated as ndg;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::node_data_generated::TokenFlags;
use tsox_frontend::ast::node_data_generated::is_parenthesized_expression;
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::{ModifierList, NodeList};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::mig::m3g_3::{is_outer_expression, OuterExpressionKinds};
use tsox_frontend::format::mig::m4o_2::{EmitHelper, Priority};

use crate::printer::NodeFactory;
use crate::mig::m4n::{HAS_COMMENT_RANGE, HAS_SOURCE_MAP_RANGE};
use tsox_core::core::text::TextRange;

pub trait EmitContextCommentRangeR36k26 {
    fn comment_range(&self, node: &Arc<Node>) -> TextRange;
    fn source_map_range(&self, node: &Arc<Node>) -> TextRange;
    fn request_emit_helper(&self, helper: &'static Arc<EmitHelper>);
    fn set_text_source(&self, node: &Arc<Node>, source: &Arc<Node>);
}

impl EmitContextCommentRangeR36k26 for crate::printer::EmitContext {
    fn comment_range(&self, node: &Arc<Node>) -> TextRange {
        if let Some(emit_node) = self.emit_nodes_try_get(node) {
            if emit_node.flags & HAS_COMMENT_RANGE != 0 {
                return emit_node.comment_range;
            }
        }
        node.loc
    }

    fn source_map_range(&self, node: &Arc<Node>) -> TextRange {
        if let Some(emit_node) = self.emit_nodes_try_get(node) {
            if emit_node.flags & HAS_SOURCE_MAP_RANGE != 0 {
                return emit_node.source_map_range;
            }
        }
        node.loc
    }

    fn request_emit_helper(&self, _helper: &'static Arc<EmitHelper>) {}

    fn set_text_source(&self, _node: &Arc<Node>, _source: &Arc<Node>) {}
}

fn prop_key_helper_arc() -> &'static Arc<EmitHelper> {
    static HELPER: OnceLock<Arc<EmitHelper>> = OnceLock::new();
    HELPER.get_or_init(|| {
        Arc::new(EmitHelper {
            name: "typescript:propKey".to_string(),
            scoped: false,
            text: r#"var __propKey = function (x) { return typeof x === "symbol" ? x : "".concat(x); };"#
                .to_string(),
            text_callback: None,
            priority: Some(Priority { value: 1 }),
            dependencies: Vec::new(),
            import_name: "__propKey".to_string(),
        })
    })
}

pub fn prop_key_helper() -> &'static EmitHelper {
    prop_key_helper_arc()
}

impl<'a> NodeFactory<'a> {
    pub fn new_function_call_call(
        &self,
        target: &Arc<Node>,
        this_arg: &Arc<Node>,
        arguments_list: &[Arc<Node>],
    ) -> Arc<Node> {
        let mut args = Vec::with_capacity(1 + arguments_list.len());
        args.push(Arc::clone(this_arg));
        args.extend(arguments_list.iter().cloned());
        self.new_call_expression(
            &self.new_property_access_expression(
                target,
                None,
                &self.new_identifier("call"),
                NodeFlags::empty(),
            ),
            None,
            None,
            self.new_node_list(args),
            NodeFlags::empty(),
        )
    }

    pub fn new_reflect_get_call(
        &self,
        target: &Arc<Node>,
        property_key: &Arc<Node>,
        receiver: &Arc<Node>,
    ) -> Arc<Node> {
        self.new_call_expression(
            &self.new_property_access_expression(
                &self.new_identifier("Reflect"),
                None,
                &self.new_identifier("get"),
                NodeFlags::empty(),
            ),
            None,
            None,
            self.new_node_list(vec![
                Arc::clone(target),
                Arc::clone(property_key),
                Arc::clone(receiver),
            ]),
            NodeFlags::empty(),
        )
    }

    pub fn new_reflect_set_call(
        &self,
        target: &Arc<Node>,
        property_key: &Arc<Node>,
        value: &Arc<Node>,
        receiver: &Arc<Node>,
    ) -> Arc<Node> {
        self.new_call_expression(
            &self.new_property_access_expression(
                &self.new_identifier("Reflect"),
                None,
                &self.new_identifier("set"),
                NodeFlags::empty(),
            ),
            None,
            None,
            self.new_node_list(vec![
                Arc::clone(target),
                Arc::clone(property_key),
                Arc::clone(value),
                Arc::clone(receiver),
            ]),
            NodeFlags::empty(),
        )
    }

    pub fn new_string_literal_from_node(&self, text_source_node: &Arc<Node>) -> Arc<Node> {
        let mut text = String::new();
        match text_source_node.kind {
            SyntaxKind::Identifier
            | SyntaxKind::PrivateIdentifier
            | SyntaxKind::JsxNamespacedName
            | SyntaxKind::StringLiteral
            | SyntaxKind::NumericLiteral
            | SyntaxKind::BigIntLiteral
            | SyntaxKind::NoSubstitutionTemplateLiteral
            | SyntaxKind::TemplateHead
            | SyntaxKind::TemplateMiddle
            | SyntaxKind::TemplateTail
            | SyntaxKind::RegularExpressionLiteral => {
                text = text_source_node.text().to_string();
            }
            _ => {}
        }
        let node = Arc::new(Node::new(
            SyntaxKind::StringLiteral,
            NodeData::StringLiteral(ndg::StringLiteralData {
                text: text.to_string(),
                token_flags: TokenFlags::default(),
            }),
        ));
        self.emit_context.set_text_source(&node, text_source_node);
        node
    }

    pub fn new_prop_key_helper(&self, expr: &Arc<Node>) -> Arc<Node> {
        self.emit_context.request_emit_helper(prop_key_helper_arc());
        self.new_call_expression(
            &self.new_unscoped_helper_name("__propKey"),
            None,
            None,
            self.new_node_list(vec![Arc::clone(expr)]),
            NodeFlags::empty(),
        )
    }

    pub fn update_tagged_template_expression(
        &self,
        node: &Arc<Node>,
        tag: &Arc<Node>,
        type_arguments: Option<Arc<NodeList>>,
        question_dot_token: Option<Arc<Node>>,
        template: &Arc<Node>,
        flags: NodeFlags,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::TaggedTemplateExpression,
            NodeData::TaggedTemplateExpression(ndg::TaggedTemplateExpressionData {
                tag: tag.clone(),
                question_dot_token,
                type_arguments,
                template: template.clone(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = flags;
        Arc::new(updated)
    }

    pub fn update_parameter_declaration(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        dot_dot_dot_token: Option<&Arc<Node>>,
        name: &Arc<Node>,
        question_token: Option<&Arc<Node>>,
        type_node: Option<&Arc<Node>>,
        initializer: Option<&Arc<Node>>,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::Parameter,
            NodeData::ParameterDeclaration(ndg::ParameterDeclarationData {
                modifiers,
                dot_dot_dot_token: dot_dot_dot_token.cloned(),
                name: name.clone(),
                question_token: question_token.cloned(),
                type_node: type_node.cloned(),
                initializer: initializer.cloned(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_for_statement(
        &self,
        node: &Arc<Node>,
        initializer: Option<&Arc<Node>>,
        condition: Option<&Arc<Node>>,
        incrementor: Option<&Arc<Node>>,
        statement: &Arc<Node>,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::ForStatement,
            NodeData::ForStatement(ndg::ForStatementData {
                initializer: initializer.cloned(),
                condition: condition.cloned(),
                incrementor: incrementor.cloned(),
                statement: statement.clone(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_computed_property_name(
        &self,
        node: &Arc<Node>,
        expression: &Arc<Node>,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::ComputedPropertyName,
            NodeData::ComputedPropertyName(ndg::ComputedPropertyNameData {
                expression: expression.clone(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_spread_element(&self, node: &Arc<Node>, expression: &Arc<Node>) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::SpreadElement,
            NodeData::SpreadElement(ndg::SpreadElementData {
                expression: expression.clone(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_spread_assignment(&self, node: &Arc<Node>, expression: &Arc<Node>) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::SpreadAssignment,
            NodeData::SpreadAssignment(ndg::SpreadAssignmentData {
                expression: expression.clone(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_array_literal_expression(
        &self,
        node: &Arc<Node>,
        elements: &[Arc<Node>],
        multi_line: bool,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::ArrayLiteralExpression,
            NodeData::ArrayLiteralExpression(ndg::ArrayLiteralExpressionData {
                elements: Arc::new(NodeList::new(elements.to_vec())),
                multi_line,
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_object_literal_expression(
        &self,
        node: &Arc<Node>,
        properties: &[Arc<Node>],
        multi_line: bool,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::ObjectLiteralExpression,
            NodeData::ObjectLiteralExpression(ndg::ObjectLiteralExpressionData {
                properties: Arc::new(NodeList::new(properties.to_vec())),
                multi_line,
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_parenthesized_expression(
        &self,
        node: &Arc<Node>,
        expression: &Arc<Node>,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::ParenthesizedExpression,
            NodeData::ParenthesizedExpression(ndg::ParenthesizedExpressionData {
                expression: expression.clone(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_partially_emitted_expression(
        &self,
        node: &Arc<Node>,
        expression: &Arc<Node>,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::PartiallyEmittedExpression,
            NodeData::PartiallyEmittedExpression(ndg::PartiallyEmittedExpressionData {
                expression: expression.clone(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn is_ignorable_paren(&self, node: &Arc<Node>) -> bool {
        fn node_is_synthesized(node: &Node) -> bool {
            (node.pos() as i32) < 0 || (node.end() as i32) < 0
        }
        fn range_is_synthesized(loc: TextRange) -> bool {
            (loc.pos() as i32) < 0 || (loc.end() as i32) < 0
        }
        is_parenthesized_expression(node)
            && node_is_synthesized(node)
            && range_is_synthesized(EmitContextCommentRangeR36k26::source_map_range(
                self.emit_context,
                node,
            ))
            && range_is_synthesized(EmitContextCommentRangeR36k26::comment_range(
                self.emit_context,
                node,
            ))
    }

    pub fn update_outer_expression(
        &self,
        outer_expression: &Arc<Node>,
        expression: &Arc<Node>,
    ) -> Arc<Node> {
        match outer_expression.kind {
            SyntaxKind::ParenthesizedExpression => {
                self.update_parenthesized_expression(outer_expression, expression)
            }
            SyntaxKind::PartiallyEmittedExpression => {
                self.update_partially_emitted_expression(outer_expression, expression)
            }
            _ => panic!(
                "Unexpected outer expression kind: {:?}",
                outer_expression.kind
            ),
        }
    }

    pub fn restore_outer_expressions(
        &self,
        outer_expression: &Arc<Node>,
        inner_expression: &Arc<Node>,
        kinds: OuterExpressionKinds,
    ) -> Arc<Node> {
        if is_outer_expression(outer_expression, kinds) && !self.is_ignorable_paren(outer_expression)
        {
            let outer_inner = outer_expression
                .expression()
                .expect("outer expression requires inner expression")
                .clone();
            return self.update_outer_expression(
                outer_expression,
                &self.restore_outer_expressions(
                    &outer_inner,
                    inner_expression,
                    OuterExpressionKinds::ALL,
                ),
            );
        }
        inner_expression.clone()
    }
}
