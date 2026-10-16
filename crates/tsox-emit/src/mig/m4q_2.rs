#![allow(dead_code, unused_imports, unused_variables)]

#[path = "r36k14_1.rs"]
pub mod r36k14_defs;

#[path = "r37k5_defs.rs"]
pub mod r37k5_defs;

#[path = "r39k07_defs.rs"]
pub mod r39k07_defs;

#[path = "r39k07b_defs.rs"]
pub mod r39k07b_defs;

use tsox_core::core::text::TextRange;
use tsox_frontend::ast::node::{ModifierList, Node, NodeList};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::*;
use super::m4q::r33k12_defs::{OperatorPrecedence, TypePrecedence, greatest_end, skip_partially_emitted_expressions, can_emit_simple_arrow_head, is_parse_tree_node};

use super::m4q::Printer;
use r36k14_defs::NodeAsExt;
use r39k07_defs::{can_emit_simple_arrow_head07, greatest_end07};
use crate::printer::GetLiteralTextFlags;
use super::m4q::r33k12_defs::{ListFlags, WriteKind};
use super::m4q::r33k12_defs::{
    LF_ALLOW_TRAILING_COMMA, LF_INDEX_SIGNATURE_PARAMETERS, LF_NAMED_IMPORTS_OR_EXPORTS_ELEMENTS,
    LF_NEW_EXPRESSION_ARGUMENTS, LF_OBJECT_BINDING_PATTERN_ELEMENTS,
    LF_OBJECT_LITERAL_EXPRESSION_PROPERTIES, LF_PARAMETERS, LF_PREFER_NEW_LINE,
    LF_SINGLE_ARROW_PARAMETER,
};

impl Printer {

    pub(crate) fn emit_named_import_bindings(&mut self, node: Option<&Node>) { ::tsox_core::fntrace::enter("emit_named_import_bindings"); 
        let Some(node) = node else { return };
        match node.kind {
            SyntaxKind::NamespaceImport => self.emit_namespace_import(node.as_namespace_import().as_node()),
            SyntaxKind::NamedImports => self.emit_named_imports(node.as_named_imports().as_node()),
            _ => panic!("unhandled NamedImportBindings: {:?}", node.kind),
        }
    }

    pub(crate) fn emit_named_imports(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_named_imports"); 
        let state = self.enter_node(node);
        let imports = node.as_named_imports();
        self.write_punctuation("{");
        self.emit_list(
            Self::emit_import_specifier_node,
            imports.as_node(),
            imports.elements(),
            LF_NAMED_IMPORTS_OR_EXPORTS_ELEMENTS,
        );
        self.write_punctuation("}");
        self.exit_node(imports.as_node(), state);
    }

    pub(crate) fn emit_named_tuple_member(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_named_tuple_member"); 
        let state = self.enter_node(node);
        let member = node.as_named_tuple_member();
        self.emit_punctuation_node(member.dot_dot_dot_token());
        self.emit_identifier_name(&member.name().as_identifier());
        self.emit_punctuation_node(member.question_token());
        self.emit_token(
            SyntaxKind::ColonToken,
            greatest_end07(member.name().end(), &[member.question_token()]),
            WriteKind::Punctuation,
            member.as_node(),
        );
        self.writer.write_space(" ");
        self.emit_type_node_outside_extends(member.type_().unwrap());
        self.exit_node(member.as_node(), state);
    }

    pub(crate) fn emit_namespace_export(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_namespace_export"); 
        let state = self.enter_node(node);
        let export = node.as_namespace_export();
        let pos = self.emit_token(SyntaxKind::AsteriskToken, export.as_node().pos(), WriteKind::Punctuation, export.as_node());
        self.writer.write_space(" ");
        self.emit_token(SyntaxKind::AsKeyword, pos, WriteKind::Keyword, export.as_node());
        self.writer.write_space(" ");
        self.emit_module_export_name(Some(export.name()));
        self.exit_node(export.as_node(), state);
    }

    pub(crate) fn emit_namespace_export_declaration(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_namespace_export_declaration"); 
        let state = self.enter_node(node);
        let decl = node.as_namespace_export_declaration();
        let pos = self.emit_token(SyntaxKind::ExportKeyword, decl.as_node().pos(), WriteKind::Keyword, decl.as_node());
        self.writer.write_space(" ");
        let pos = self.emit_token(SyntaxKind::AsKeyword, pos, WriteKind::Keyword, decl.as_node());
        self.writer.write_space(" ");
        self.emit_token(SyntaxKind::NamespaceKeyword, pos, WriteKind::Keyword, decl.as_node());
        self.writer.write_space(" ");
        self.emit_binding_identifier(&decl.name().as_identifier());
        self.write_trailing_semicolon();
        self.exit_node(decl.as_node(), state);
    }

    pub(crate) fn emit_namespace_import(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_namespace_import"); 
        let state = self.enter_node(node);
        let import = node.as_namespace_import();
        let pos = self.emit_token(SyntaxKind::AsteriskToken, import.as_node().pos(), WriteKind::Punctuation, import.as_node());
        self.writer.write_space(" ");
        self.emit_token(SyntaxKind::AsKeyword, pos, WriteKind::Keyword, import.as_node());
        self.writer.write_space(" ");
        self.emit_binding_identifier(&import.name().as_identifier());
        self.exit_node(import.as_node(), state);
    }

    pub(crate) fn emit_nested_module_name(&mut self, node: Option<&Node>) { ::tsox_core::fntrace::enter("emit_nested_module_name"); 
        let Some(node) = node else { return };
        match node.kind {
            SyntaxKind::Identifier => self.emit_identifier_name(&node.as_identifier()),
            SyntaxKind::StringLiteral => self.emit_string_literal(node.as_string_literal().as_node()),
            _ => panic!("unexpected ModuleName: {:?}", node.kind),
        }
    }

    pub(crate) fn emit_parameter_name(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_parameter_name"); 
        let saved_write_kind = self.write_kind;
        self.write_kind = WriteKind::Parameter;
        self.emit_binding_name(node);
        self.write_kind = saved_write_kind;
    }

    pub(crate) fn emit_parameter(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_parameter"); 
        let state = self.enter_node(node);
        let parameter = node.as_parameter_declaration();
        self.emit_modifier_list(parameter.as_node(), parameter.modifiers(), true);
        self.emit_token_node(parameter.dot_dot_dot_token());
        self.emit_parameter_name(parameter.name());
        self.emit_token_node(parameter.question_token());
        self.emit_type_annotation(parameter.type_());
        self.emit_initializer(
            parameter.initializer(),
            greatest_end07(
                parameter.as_node().pos(),
                &[parameter.type_(), parameter.question_token(), Some(parameter.name())],
            ),
            parameter.as_node(),
        );
        self.exit_node(parameter.as_node(), state);
    }

    pub(crate) fn emit_parameter_declaration_node(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_parameter_declaration_node"); 
        self.emit_parameter(node.as_parameter_declaration().as_node());
    }

    pub(crate) fn emit_parameters(&mut self, parent_node: &Node, parameters: &NodeList) { ::tsox_core::fntrace::enter("emit_parameters"); 
        self.generate_all_names(parameters);
        self.emit_list(Self::emit_parameter_declaration_node, parent_node, parameters, LF_PARAMETERS);
    }
    pub(crate) fn emit_new_expression(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_new_expression"); 
        let state = self.enter_node(node);
        let new_expr = node.as_new_expression();
        self.emit_token(SyntaxKind::NewKeyword, new_expr.as_node().pos(), WriteKind::Keyword, new_expr.as_node());
        self.writer.write_space(" ");
        if skip_partially_emitted_expressions(new_expr.expression().unwrap()).kind == SyntaxKind::CallExpression {
            self.emit_expression(new_expr.expression().unwrap(), OperatorPrecedence::Parentheses);
        } else {
            self.emit_expression(new_expr.expression().unwrap(), OperatorPrecedence::Member);
        }
        self.emit_type_arguments(new_expr.as_node(), new_expr.type_arguments());
        if let Some(arguments) = new_expr.arguments() {
            self.emit_list(Self::emit_argument, new_expr.as_node(), arguments, LF_NEW_EXPRESSION_ARGUMENTS);
        }
        self.exit_node(new_expr.as_node(), state);
    }

    pub(crate) fn emit_no_substitution_template_literal(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_no_substitution_template_literal"); 
        let state = self.enter_node(node);
        self.emit_literal(node, GetLiteralTextFlags::NONE);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_non_null_expression(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_non_null_expression"); 
        let state = self.enter_node(node);
        let non_null = node.as_non_null_expression();
        self.emit_expression(non_null.expression().unwrap(), OperatorPrecedence::Member);
        self.write_operator("!");
        self.exit_node(non_null.as_node(), state);
    }

    pub(crate) fn emit_not_emitted_statement(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_not_emitted_statement"); 
        let state = self.enter_node(node);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_not_emitted_type_element(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_not_emitted_type_element"); 
        let state = self.enter_node(node);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_numeric_literal(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_numeric_literal"); 
        let state = self.enter_node(node);
        self.emit_literal(node, GetLiteralTextFlags::NONE);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_object_binding_pattern(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_object_binding_pattern"); 
        let state = self.enter_node(node);
        let pattern = node.as_binding_pattern();
        self.write_punctuation("{");
        self.emit_list(Self::emit_binding_element_node, pattern.as_node(), pattern.elements(), LF_OBJECT_BINDING_PATTERN_ELEMENTS);
        self.write_punctuation("}");
        self.exit_node(pattern.as_node(), state);
    }

    pub(crate) fn emit_object_literal_element(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_object_literal_element"); 
        match node.kind {
            SyntaxKind::PropertyAssignment => self.emit_property_assignment(node.as_property_assignment().as_node()),
            SyntaxKind::ShorthandPropertyAssignment => {
                self.emit_shorthand_property_assignment(node.as_shorthand_property_assignment().as_node())
            }
            SyntaxKind::SpreadAssignment => self.emit_spread_assignment(node.as_spread_assignment().as_node()),
            SyntaxKind::MethodDeclaration => self.emit_method_declaration(node.as_method_declaration().as_node()),
            SyntaxKind::GetAccessor => self.emit_get_accessor_declaration(node.as_get_accessor_declaration().as_node()),
            SyntaxKind::SetAccessor => self.emit_set_accessor_declaration(node.as_set_accessor_declaration().as_node()),
            _ => panic!("unhandled ObjectLiteralElement: {:?}", node.kind),
        }
    }

    pub(crate) fn emit_object_literal_expression(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_object_literal_expression"); 
        let state = self.enter_node(node);
        let literal = node.as_object_literal_expression();
        let indented = self.should_emit_indented(literal.as_node());
        self.increase_indent_if(indented);
        self.push_name_generation_scope(literal.as_node());
        self.generate_all_member_names(literal.properties());
        let mut format = LF_OBJECT_LITERAL_EXPRESSION_PROPERTIES;
        if literal.multi_line() {
            format |= LF_PREFER_NEW_LINE;
        }
        if self.should_allow_trailing_comma(literal.as_node(), literal.properties()) {
            format |= LF_ALLOW_TRAILING_COMMA;
        }
        self.emit_list(Self::emit_object_literal_element, literal.as_node(), literal.properties(), format);
        self.pop_name_generation_scope(literal.as_node());
        self.decrease_indent_if(indented);
        self.exit_node(literal.as_node(), state);
    }

    pub(crate) fn emit_omitted_expression(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_omitted_expression"); 
        let state = self.enter_node(node);
        self.exit_node(node, state);
    }

    pub(crate) fn emit_optional_type(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_optional_type"); 
        let state = self.enter_node(node);
        let optional = node.as_optional_type_node();
        self.emit_postfix_type_operand(optional.type_().unwrap(), optional.as_node());
        self.write_punctuation("?");
        self.exit_node(optional.as_node(), state);
    }

    pub(crate) fn emit_parameters_for_arrow(&mut self, parent_node: &Node, parameters: &NodeList) { ::tsox_core::fntrace::enter("emit_parameters_for_arrow"); 
        if can_emit_simple_arrow_head07(parent_node, parameters) {
            self.generate_all_names(parameters);
            self.emit_list(Self::emit_parameter_declaration_node, parent_node, parameters, LF_SINGLE_ARROW_PARAMETER);
        } else {
            self.emit_parameters(parent_node, parameters);
        }
    }

    pub(crate) fn emit_parameters_for_index_signature(&mut self, parent_node: &Node, parameters: &NodeList) { ::tsox_core::fntrace::enter("emit_parameters_for_index_signature"); 
        self.generate_all_names(parameters);
        self.emit_list(Self::emit_parameter_declaration_node, parent_node, parameters, LF_INDEX_SIGNATURE_PARAMETERS);
    }

    pub(crate) fn emit_parenthesized_expression(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_parenthesized_expression"); 
        let state = self.enter_node(node);
        let paren = node.as_parenthesized_expression();
        let open_paren_pos = self.emit_token(SyntaxKind::OpenParenToken, paren.as_node().pos(), WriteKind::Punctuation, paren.as_node());
        let indented = self.write_line_separators_and_indent_before(paren.expression().map(|e| e.as_ref()), paren.as_node());
        self.emit_expression(paren.expression().unwrap(), OperatorPrecedence::Comma);
        self.write_line_separators_after(paren.expression().map(|e| e.as_ref()), paren.as_node());
        self.decrease_indent_if(indented);
        let close_paren_pos = match paren.expression() {
            Some(expression) => expression.end(),
            None => open_paren_pos,
        };
        self.emit_token(SyntaxKind::CloseParenToken, close_paren_pos, WriteKind::Punctuation, paren.as_node());
        self.exit_node(paren.as_node(), state);
    }

    pub(crate) fn emit_parenthesized_type(&mut self, node: &Node) { ::tsox_core::fntrace::enter("emit_parenthesized_type"); 
        let state = self.enter_node(node);
        let paren = node.as_parenthesized_type_node();
        self.write_punctuation("(");
        self.emit_type_node_outside_extends(paren.type_().unwrap());
        self.write_punctuation(")");
        self.exit_node(paren.as_node(), state);
    }

    pub(crate) fn emit_postfix_type_operand(&mut self, operand: &Node, parent: &Node) { ::tsox_core::fntrace::enter("emit_postfix_type_operand"); 
        if is_parse_tree_node(parent) && operand.kind == SyntaxKind::TypeQuery {
            self.emit_type_node(operand, TypePrecedence::TypeOperator);
            return;
        }
        self.emit_type_node(operand, TypePrecedence::Postfix);
    }
}
