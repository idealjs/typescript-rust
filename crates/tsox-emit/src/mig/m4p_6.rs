#![allow(dead_code, unused_imports, unused_variables)]

#[path = "r36k20_jsx_defs.rs"]
pub mod r36k20_jsx_defs;

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::node_is_synthesized;
use tsox_frontend::format::mig::m4t_3::get_lines_between_positions;
use tsox_frontend::format::mig::m4t_4::greatest_end;

use tsox_frontend::format::mig::m4o::ListFormat;
use tsox_frontend::format::mig::m4o_2::WriteKind;
use tsox_frontend::ast::mig::m3e_3::{
    OperatorPrecedence, OPERATOR_PRECEDENCE_DISALLOW_COMMA, OPERATOR_PRECEDENCE_LOWEST,
};

use r36k20_jsx_defs::JsxNodeExt;

use super::m4p::Printer;

const JSX_ELEMENT_OR_FRAGMENT_CHILDREN: ListFormat =
    ListFormat(crate::mig::m4q::r33k12_defs::LF_JSX_ELEMENT_OR_FRAGMENT_CHILDREN);
const JSX_ELEMENT_ATTRIBUTES: ListFormat =
    ListFormat(crate::mig::m4q::r33k12_defs::LF_JSX_ELEMENT_ATTRIBUTES);

fn greatest_node_end(end: usize, nodes: &[Option<&Arc<Node>>]) -> usize { ::tsox_core::fntrace::enter("greatest_node_end"); 
    let mut end = end;
    for node in nodes.iter().rev() {
        if let Some(node_end) = node.map(|n| n.end()) {
            if end < node_end {
                end = node_end;
            }
        }
    }
    end
}

impl Printer {
    pub fn emit_jsx_element(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_element"); 
        let state = self.enter_node(node);
        self.emit_jsx_opening_element(node.opening_element());
        self.emit_list(
            Self::emit_jsx_child,
            node,
            node.children(),
            JSX_ELEMENT_OR_FRAGMENT_CHILDREN,
        );
        self.emit_jsx_closing_element(node.closing_element());
        self.exit_node(node, state);
    }

    pub fn emit_jsx_self_closing_element(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_self_closing_element"); 
        let state = self.enter_node(node);
        self.write_punctuation("<");
        self.emit_jsx_tag_name(node.tag_name());
        self.emit_type_arguments(node, node.type_arguments());
        self.write_space();
        self.emit_jsx_attributes(node.attributes());
        self.write_punctuation("/>");
        self.exit_node(node, state);
    }

    pub fn emit_jsx_fragment(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_fragment"); 
        let state = self.enter_node(node);
        self.emit_jsx_opening_fragment(node.opening_fragment());
        self.emit_list(
            Self::emit_jsx_child,
            node,
            node.children(),
            JSX_ELEMENT_OR_FRAGMENT_CHILDREN,
        );
        self.emit_jsx_closing_fragment(node.closing_fragment());
        self.exit_node(node, state);
    }

    pub fn emit_jsx_opening_element(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_opening_element"); 
        let state = self.enter_node(node);
        self.write_punctuation("<");
        let attributes = node.attributes();
        let indented =
            self.write_line_separators_and_indent_before(node.tag_name(), node);        self.emit_jsx_tag_name(node.tag_name());
        self.emit_type_arguments(node, node.type_arguments());
        if !attributes.properties().is_empty() {
            self.write_space();
        }
        self.emit_jsx_attributes(attributes);
        self.write_line_separators_after(attributes, node);
        self.decrease_indent_if(indented > 0);
        self.write_punctuation(">");
        self.exit_node(node, state);
    }

    pub fn emit_jsx_closing_element(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_closing_element"); 
        let state = self.enter_node(node);
        self.write_punctuation("</");
        self.emit_jsx_tag_name(node.tag_name());
        self.write_punctuation(">");
        self.exit_node(node, state);
    }

    pub fn emit_jsx_opening_fragment(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_opening_fragment"); 
        let state = self.enter_node(node);
        self.write_punctuation("<");
        self.write_punctuation(">");
        self.exit_node(node, state);
    }

    pub fn emit_jsx_closing_fragment(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_closing_fragment"); 
        let state = self.enter_node(node);
        self.write_punctuation("</");
        self.write_punctuation(">");
        self.exit_node(node, state);
    }

    pub fn emit_jsx_text(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_text"); 
        let state = self.enter_node(node);
        self.write_literal(node.text());
        self.exit_node(node, state);
    }

    pub fn emit_jsx_attributes(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_attributes"); 
        let state = self.enter_node(node);
        self.emit_list(
            Self::emit_jsx_attribute_like,
            node,
            node.properties(),
            JSX_ELEMENT_ATTRIBUTES,
        );
        self.exit_node(node, state);
    }

    pub fn emit_jsx_attribute(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_attribute"); 
        let state = self.enter_node(node);
        self.emit_jsx_attribute_name(node.name().unwrap());
        if let Some(initializer) = node.jsx_attribute_initializer() {
            self.write_punctuation("=");
            self.emit_jsx_attribute_value(initializer);
        }
        self.exit_node(node, state);
    }

    pub fn emit_jsx_spread_attribute(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_spread_attribute"); 
        let state = self.enter_node(node);
        self.write_punctuation("{...");
        self.emit_expression(node.expression().unwrap(), OPERATOR_PRECEDENCE_LOWEST);
        self.write_punctuation("}");
        self.exit_node(node, state);
    }

    pub fn emit_jsx_attribute_like(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_attribute_like"); 
        match node.kind {
            SyntaxKind::JsxAttribute => self.emit_jsx_attribute(node),
            SyntaxKind::JsxSpreadAttribute => self.emit_jsx_spread_attribute(node),
            _ => panic!("unhandled JsxAttributeLike: {:?}", node.kind),
        }
    }

    pub fn emit_jsx_expression(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_expression"); 
        let state = self.enter_node(node);
        let expression = node.expression();
        if expression.is_some()
            || !self.comments_disabled
                && !node_is_synthesized(node)
                && self.has_comments_at_position(node.pos())
        {
            let indented = self.current_source_file.is_some()
                && !node_is_synthesized(node)
                && get_lines_between_positions(
                    self.current_source_file.as_deref().unwrap(),
                    node.pos() as i64,
                    node.end() as i64,
                )
                    != 0;
            self.increase_indent_if(indented);
            let end =
                self.emit_token(SyntaxKind::OpenBraceToken, node.pos(), WriteKind::Punctuation, node);
            self.emit_token_node(node.dot_dot_dot_token());
            if let Some(expression) = expression {
                self.emit_expression(expression, OPERATOR_PRECEDENCE_DISALLOW_COMMA);
            }
            self.emit_token(
                SyntaxKind::CloseBraceToken,
                greatest_node_end(end, &[expression, node.dot_dot_dot_token()]),
                WriteKind::Punctuation,
                node,
            );
            self.decrease_indent_if(indented);
        }
        self.exit_node(node, state);
    }

    pub fn emit_jsx_namespaced_name(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_namespaced_name"); 
        let state = self.enter_node(node);
        self.emit_identifier_name(node.namespace());
        self.write_punctuation(":");
        self.emit_identifier_name(node.name().unwrap());
        self.exit_node(node, state);
    }

    pub fn emit_jsx_child(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_child"); 
        match node.kind {
            SyntaxKind::JsxText => self.emit_jsx_text(node),
            SyntaxKind::JsxExpression => self.emit_jsx_expression(node),
            SyntaxKind::JsxElement => self.emit_jsx_element(node),
            SyntaxKind::JsxSelfClosingElement => self.emit_jsx_self_closing_element(node),
            SyntaxKind::JsxFragment => self.emit_jsx_fragment(node),
            _ => panic!("unhandled JsxChild: {:?}", node.kind),
        }
    }

    pub fn emit_jsx_tag_name(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_tag_name"); 
        match node.kind {
            SyntaxKind::Identifier => self.emit_identifier_reference(node),
            SyntaxKind::ThisKeyword => self.emit_keyword_expression(node),
            SyntaxKind::JsxNamespacedName => self.emit_jsx_namespaced_name(node),
            SyntaxKind::PropertyAccessExpression => self.emit_property_access_expression(node),
            _ => panic!("unhandled JsxTagName: {:?}", node.kind),
        }
    }

    pub fn emit_jsx_attribute_name(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_attribute_name"); 
        match node.kind {
            SyntaxKind::Identifier => self.emit_identifier_name(node),
            SyntaxKind::JsxNamespacedName => self.emit_jsx_namespaced_name(node),
            _ => panic!("unhandled JsxAttributeName: {:?}", node.kind),
        }
    }

    pub fn emit_jsx_attribute_value(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_jsx_attribute_value"); 
        match node.kind {
            SyntaxKind::StringLiteral => self.emit_string_literal(node),
            SyntaxKind::JsxExpression => self.emit_jsx_expression(node),
            SyntaxKind::JsxElement => self.emit_jsx_element(node),
            SyntaxKind::JsxSelfClosingElement => self.emit_jsx_self_closing_element(node),
            SyntaxKind::JsxFragment => self.emit_jsx_fragment(node),
            _ => self.emit_expression(node, OPERATOR_PRECEDENCE_LOWEST),
        }
    }

    pub fn emit_external_module_reference(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_external_module_reference"); 
        let state = self.enter_node(node);
        self.write_keyword("require");
        self.write_punctuation("(");
        self.emit_expression(node.expression().unwrap(), OPERATOR_PRECEDENCE_DISALLOW_COMMA);
        self.write_punctuation(")");
        self.exit_node(node, state);
    }

    pub fn emit_import_attribute_name(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_import_attribute_name"); 
        match node.kind {
            SyntaxKind::Identifier => self.emit_identifier_name(node),
            SyntaxKind::StringLiteral => self.emit_string_literal(node),
            _ => panic!("unexpected ImportAttributeName: {:?}", node.kind),
        }
    }
}
