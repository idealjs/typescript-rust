use std::sync::Arc;

use crate::ast::node::Node;
use crate::ast::node_data_generated::{
    NodeData, is_array_literal_expression, is_external_module_reference, is_identifier,
    is_object_literal_expression, is_private_identifier,
};
use crate::ast::node_flags::ModifierFlags;
use super::m3c_3::{
    compute_subtree_facts_enum_declaration, compute_subtree_facts_enum_member,
    compute_subtree_facts_export_assignment, compute_subtree_facts_export_declaration,
    compute_subtree_facts_export_specifier, compute_subtree_facts_expression_with_type_arguments,
    compute_subtree_facts_for_in_of_statement, compute_subtree_facts_function_declaration,
    compute_subtree_facts_function_expression, compute_subtree_facts_heritage_clause,
    compute_subtree_facts_import_clause, compute_subtree_facts_import_equals_declaration,
    compute_subtree_facts_import_specifier, compute_subtree_facts_jsx_attribute,
    compute_subtree_facts_jsx_attributes, compute_subtree_facts_jsx_closing_element,
    compute_subtree_facts_jsx_element, compute_subtree_facts_jsx_expression,
    compute_subtree_facts_jsx_fragment, compute_subtree_facts_jsx_namespaced_name,
    compute_subtree_facts_jsx_opening_element, compute_subtree_facts_jsx_self_closing_element,
    compute_subtree_facts_jsx_spread_attribute,
};
use super::m3c_4::{
    compute_subtree_facts_keyword_expression, compute_subtree_facts_meta_property,
    compute_subtree_facts_method_declaration, compute_subtree_facts_module_declaration,
    compute_subtree_facts_new_expression, compute_subtree_facts_no_substitution_template_literal,
    compute_subtree_facts_non_null_expression, compute_subtree_facts_parameter_declaration,
    compute_subtree_facts_property_access_expression, compute_subtree_facts_property_assignment,
    compute_subtree_facts_property_declaration, compute_subtree_facts_return_statement,
    compute_subtree_facts_satisfies_expression, compute_subtree_facts_shorthand_property_assignment,
};
use super::m3d::*;
use crate::ast::subtree_facts::{
    SubtreeContainsAnyAwait, SubtreeContainsAwait, SubtreeContainsClassFields,
    SubtreeContainsDecorators, SubtreeContainsDynamicImport,
    SubtreeContainsEsObjectRestOrSpread as SubtreeContainsESObjectRestOrSpread,
    SubtreeContainsForAwaitOrAsyncGenerator, SubtreeContainsIdentifier, SubtreeContainsJsx,
    SubtreeContainsMissingCatchClauseVariable, SubtreeContainsObjectRestOrSpread,
    SubtreeContainsPrivateIdentifierInExpression, SubtreeContainsRestOrSpread,
    SubtreeContainsTypeScript,
};
use super::w2::contains_object_rest_or_spread;
use crate::ast::syntax_kind_generated::SyntaxKind;
use crate::ast::utilities_misc::is_this_identifier;
use crate::scanner::error_callback::TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE;

pub fn subtree_facts(node: &Node) -> SubtreeFacts {
    compute_subtree_facts(node)
}

pub fn compute_subtree_facts(node: &Node) -> SubtreeFacts {
    match &node.data {
        NodeData::GetAccessorDeclaration(d) => {
            compute_subtree_facts_accessor_declaration_base(
                &d.modifiers,
                Some(&d.name),
                d.type_parameters.as_ref(),
                &d.parameters,
                d.type_node.as_ref(),
                d.full_signature.as_ref(),
                d.body.as_ref(),
            )
        }
        NodeData::SetAccessorDeclaration(d) => {
            compute_subtree_facts_accessor_declaration_base(
                &d.modifiers,
                Some(&d.name),
                d.type_parameters.as_ref(),
                &d.parameters,
                d.type_node.as_ref(),
                d.full_signature.as_ref(),
                d.body.as_ref(),
            )
        }
        NodeData::ArrowFunction(d) => compute_subtree_facts_arrow_function(d),
        NodeData::AsExpression(d) => compute_subtree_facts_as_expression(d),
        NodeData::AwaitExpression(d) => compute_subtree_facts_await_expression(d),
        NodeData::BigIntLiteral(_) => SubtreeFactsNone,
        NodeData::BinaryExpression(d) => compute_subtree_facts_binary_expression(d),
        NodeData::BindingElement(d) => compute_subtree_facts_binding_element(d),
        NodeData::BindingPattern(d) => compute_subtree_facts_binding_pattern(node.kind, d),
        NodeData::CallExpression(d) => compute_subtree_facts_call_expression(d),
        NodeData::CatchClause(d) => compute_subtree_facts_catch_clause(d),
        NodeData::ClassDeclaration(d) => compute_subtree_facts_class_like_base(
            &d.modifiers,
            d.name.as_ref(),
            d.type_parameters.as_ref(),
            d.heritage_clauses.as_ref(),
            &d.members,
        ),
        NodeData::ClassExpression(d) => compute_subtree_facts_class_like_base(
            &d.modifiers,
            d.name.as_ref(),
            d.type_parameters.as_ref(),
            d.heritage_clauses.as_ref(),
            &d.members,
        ),
        NodeData::ClassStaticBlockDeclaration(d) => {
            compute_subtree_facts_class_static_block_declaration(d)
        }
        NodeData::ConstructorDeclaration(d) => compute_subtree_facts_constructor_declaration(d),
        NodeData::Decorator(d) => compute_subtree_facts_decorator(d),
        NodeData::EnumDeclaration(d) => compute_subtree_facts_enum_declaration(d),
        NodeData::EnumMember(d) => compute_subtree_facts_enum_member(d),
        NodeData::ExportAssignment(d) => compute_subtree_facts_export_assignment(d),
        NodeData::ExportDeclaration(d) => compute_subtree_facts_export_declaration(d),
        NodeData::ExportSpecifier(d) => compute_subtree_facts_export_specifier(d),
        NodeData::ExpressionWithTypeArguments(d) => {
            compute_subtree_facts_expression_with_type_arguments(d)
        }
        NodeData::ForInOrOfStatement(d) => compute_subtree_facts_for_in_of_statement(d),
        NodeData::FunctionDeclaration(d) => compute_subtree_facts_function_declaration(d),
        NodeData::FunctionExpression(d) => compute_subtree_facts_function_expression(d),
        NodeData::HeritageClause(d) => compute_subtree_facts_heritage_clause(d),
        NodeData::Identifier(_) => SubtreeContainsIdentifier,
        NodeData::PrivateIdentifier(_) => SubtreeContainsClassFields,
        NodeData::ImportClause(d) => compute_subtree_facts_import_clause(d),
        NodeData::ImportEqualsDeclaration(d) => compute_subtree_facts_import_equals_declaration(d),
        NodeData::ImportSpecifier(d) => compute_subtree_facts_import_specifier(d),
        NodeData::JsxAttribute(d) => compute_subtree_facts_jsx_attribute(d),
        NodeData::JsxAttributes(d) => compute_subtree_facts_jsx_attributes(d),
        NodeData::JsxClosingElement(d) => compute_subtree_facts_jsx_closing_element(d),
        NodeData::JsxClosingFragment => SubtreeContainsJsx,
        NodeData::JsxElement(d) => compute_subtree_facts_jsx_element(d),
        NodeData::JsxExpression(d) => compute_subtree_facts_jsx_expression(d),
        NodeData::JsxFragment(d) => compute_subtree_facts_jsx_fragment(d),
        NodeData::JsxNamespacedName(d) => compute_subtree_facts_jsx_namespaced_name(d),
        NodeData::JsxOpeningElement(d) => compute_subtree_facts_jsx_opening_element(d),
        NodeData::JsxOpeningFragment => SubtreeContainsJsx,
        NodeData::JsxSelfClosingElement(d) => compute_subtree_facts_jsx_self_closing_element(d),
        NodeData::JsxSpreadAttribute(d) => compute_subtree_facts_jsx_spread_attribute(d),
        NodeData::JsxText(_) => SubtreeContainsJsx,
        NodeData::KeywordExpression => compute_subtree_facts_keyword_expression(node.kind),
        NodeData::MetaProperty(d) => compute_subtree_facts_meta_property(d),
        NodeData::MethodDeclaration(d) => compute_subtree_facts_method_declaration(d),
        NodeData::ModuleDeclaration(d) => compute_subtree_facts_module_declaration(d),
        NodeData::NewExpression(d) => compute_subtree_facts_new_expression(d),
        NodeData::NoSubstitutionTemplateLiteral(d) => {
            compute_subtree_facts_no_substitution_template_literal(d)
        }
        NodeData::NonNullExpression(d) => compute_subtree_facts_non_null_expression(d),
        NodeData::ParameterDeclaration(d) => compute_subtree_facts_parameter_declaration(d),
        NodeData::PropertyAccessExpression(d) => {
            compute_subtree_facts_property_access_expression(d)
        }
        NodeData::PropertyAssignment(d) => compute_subtree_facts_property_assignment(d),
        NodeData::PropertyDeclaration(d) => compute_subtree_facts_property_declaration(d),
        NodeData::ReturnStatement(d) => compute_subtree_facts_return_statement(d),
        NodeData::SatisfiesExpression(d) => compute_subtree_facts_satisfies_expression(d),
        NodeData::ShorthandPropertyAssignment(d) => {
            compute_subtree_facts_shorthand_property_assignment(d)
        }
        _ => compute_subtree_facts_node_default(),
    }
}

fn compute_subtree_facts_node_default() -> SubtreeFacts {
    SubtreeFactsNone
}

fn compute_subtree_facts_composite_base() -> SubtreeFacts {
    unreachable!("computeSubtreeFacts must be implemented by the concrete node type")
}

pub(crate) fn modifier_flags_of(modifiers: &Option<Arc<crate::ast::node::ModifierList>>) -> ModifierFlags {
    modifiers.as_ref().map(|m| m.modifier_flags).unwrap_or_default()
}

pub(crate) fn propagate_eraseable_syntax_list_subtree_facts(
    children: Option<&Arc<crate::ast::node::NodeList>>,
) -> SubtreeFacts {
    if children.is_some() {
        SubtreeContainsTypeScript
    } else {
        SubtreeFactsNone
    }
}

pub(crate) fn propagate_eraseable_syntax_subtree_facts(child: Option<&Arc<Node>>) -> SubtreeFacts {
    if child.is_some() {
        SubtreeContainsTypeScript
    } else {
        SubtreeFactsNone
    }
}

pub(crate) fn propagate_object_binding_element_subtree_facts(child: &Arc<Node>) -> SubtreeFacts {
    let mut facts = propagate_subtree_facts(Some(child));
    if facts.contains(SubtreeContainsRestOrSpread) {
        facts = facts.without(SubtreeContainsRestOrSpread);
        facts |= SubtreeContainsObjectRestOrSpread | SubtreeContainsESObjectRestOrSpread;
    }
    facts
}

pub(crate) fn propagate_binding_element_subtree_facts(child: &Arc<Node>) -> SubtreeFacts {
    propagate_subtree_facts(Some(child)).without(SubtreeContainsRestOrSpread)
}

pub(crate) fn propagate_subtree_facts(child: Option<&Arc<Node>>) -> SubtreeFacts {
    child
        .map(|child| child.propagate_subtree_facts())
        .unwrap_or(SubtreeFactsNone)
}

pub(crate) fn propagate_node_list_subtree_facts(
    children: Option<&Arc<crate::ast::node::NodeList>>,
    propagate: impl Fn(&Arc<Node>) -> SubtreeFacts,
) -> SubtreeFacts {
    let Some(children) = children else {
        return SubtreeFactsNone;
    };
    let mut facts = SubtreeFactsNone;
    for child in &children.nodes {
        facts |= propagate(child);
    }
    facts
}

pub(crate) fn propagate_modifier_list_subtree_facts(
    children: Option<&Arc<crate::ast::node::ModifierList>>,
) -> SubtreeFacts {
    let Some(children) = children else {
        return SubtreeFactsNone;
    };
    let mut facts = SubtreeFactsNone;
    for child in &children.list.nodes {
        facts |= propagate_subtree_facts(Some(child));
    }
    facts
}

fn compute_subtree_facts_binary_expression(
    d: &crate::ast::node_data_generated::BinaryExpressionData,
) -> SubtreeFacts {
    let mut facts = propagate_modifier_list_subtree_facts(d.modifiers.as_ref())
        | propagate_subtree_facts(Some(&d.left))
        | propagate_eraseable_syntax_subtree_facts(d.type_node.as_ref())
        | propagate_subtree_facts(Some(&d.operator_token))
        | propagate_subtree_facts(Some(&d.right))
        | if d.operator_token.kind == SyntaxKind::InKeyword && is_private_identifier(&d.left) {
            SubtreeContainsClassFields | SubtreeContainsPrivateIdentifierInExpression
        } else {
            SubtreeFactsNone
        };
    if d.operator_token.kind == SyntaxKind::EqualsToken
        && (is_object_literal_expression(&d.left) || is_array_literal_expression(&d.left))
        && contains_object_rest_or_spread(&d.left)
    {
        facts |= SubtreeContainsObjectRestOrSpread;
    }
    facts
}

fn compute_subtree_facts_accessor_declaration_base(
    modifiers: &Option<Arc<crate::ast::node::ModifierList>>,
    name: Option<&Arc<Node>>,
    type_parameters: Option<&Arc<crate::ast::node::NodeList>>,
    parameters: &Arc<crate::ast::node::NodeList>,
    type_node: Option<&Arc<Node>>,
    full_signature: Option<&Arc<Node>>,
    body: Option<&Arc<Node>>,
) -> SubtreeFacts {
    if body.is_none() {
        return SubtreeContainsTypeScript;
    }
    propagate_modifier_list_subtree_facts(modifiers.as_ref())
        | propagate_subtree_facts(name)
        | propagate_eraseable_syntax_list_subtree_facts(type_parameters)
        | propagate_node_list_subtree_facts(Some(parameters), |n| propagate_subtree_facts(Some(n)))
        | propagate_eraseable_syntax_subtree_facts(type_node)
        | propagate_eraseable_syntax_subtree_facts(full_signature)
        | propagate_subtree_facts(body)
}

fn compute_subtree_facts_arrow_function(d: &crate::ast::node_data_generated::ArrowFunctionData) -> SubtreeFacts {
    propagate_modifier_list_subtree_facts(d.modifiers.as_ref())
        | propagate_eraseable_syntax_list_subtree_facts(d.type_parameters.as_ref())
        | propagate_node_list_subtree_facts(Some(&d.parameters), |n| propagate_subtree_facts(Some(n)))
        | propagate_eraseable_syntax_subtree_facts(d.type_node.as_ref())
        | propagate_eraseable_syntax_subtree_facts(d.full_signature.as_ref())
        | propagate_subtree_facts(Some(&d.body))
        | if modifier_flags_of(&d.modifiers).contains(ModifierFlags::Async) {
            SubtreeContainsAnyAwait
        } else {
            SubtreeFactsNone
        }
}

fn compute_subtree_facts_as_expression(
    d: &crate::ast::node_data_generated::AsExpressionData,
) -> SubtreeFacts {
    propagate_subtree_facts(Some(&d.expression)) | SubtreeContainsTypeScript
}

fn compute_subtree_facts_await_expression(
    d: &crate::ast::node_data_generated::AwaitExpressionData,
) -> SubtreeFacts {
    propagate_subtree_facts(Some(&d.expression))
        | SubtreeContainsAwait
        | SubtreeContainsAnyAwait
        | SubtreeContainsForAwaitOrAsyncGenerator
}

fn compute_subtree_facts_binding_element(
    d: &crate::ast::node_data_generated::BindingElementData,
) -> SubtreeFacts {
    propagate_subtree_facts(d.property_name.as_ref())
        | propagate_subtree_facts(d.name.as_ref())
        | propagate_subtree_facts(d.initializer.as_ref())
        | if d.dot_dot_dot_token.is_some() {
            SubtreeContainsRestOrSpread
        } else {
            SubtreeFactsNone
        }
}

fn compute_subtree_facts_binding_pattern(
    kind: SyntaxKind,
    d: &crate::ast::node_data_generated::BindingPatternData,
) -> SubtreeFacts {
    match kind {
        SyntaxKind::ObjectBindingPattern => propagate_node_list_subtree_facts(
            Some(&d.elements),
            propagate_object_binding_element_subtree_facts,
        ),
        SyntaxKind::ArrayBindingPattern => propagate_node_list_subtree_facts(
            Some(&d.elements),
            propagate_binding_element_subtree_facts,
        ),
        _ => SubtreeFactsNone,
    }
}

fn compute_subtree_facts_call_expression(
    d: &crate::ast::node_data_generated::CallExpressionData,
) -> SubtreeFacts {
    propagate_subtree_facts(Some(&d.expression))
        | propagate_subtree_facts(d.question_dot_token.as_ref())
        | propagate_eraseable_syntax_list_subtree_facts(d.type_arguments.as_ref())
        | propagate_node_list_subtree_facts(Some(&d.arguments), |n| propagate_subtree_facts(Some(n)))
        | if d.expression.kind == SyntaxKind::ImportKeyword {
            SubtreeContainsDynamicImport
        } else {
            SubtreeFactsNone
        }
}

fn compute_subtree_facts_catch_clause(
    d: &crate::ast::node_data_generated::CatchClauseData,
) -> SubtreeFacts {
    let mut res = propagate_subtree_facts(d.variable_declaration.as_ref())
        | propagate_subtree_facts(Some(&d.block));
    if d.variable_declaration.is_none() {
        res |= SubtreeContainsMissingCatchClauseVariable;
    }
    res
}

fn compute_subtree_facts_class_like_base(
    modifiers: &Option<Arc<crate::ast::node::ModifierList>>,
    name: Option<&Arc<Node>>,
    type_parameters: Option<&Arc<crate::ast::node::NodeList>>,
    heritage_clauses: Option<&Arc<crate::ast::node::NodeList>>,
    members: &Arc<crate::ast::node::NodeList>,
) -> SubtreeFacts {
    if modifier_flags_of(modifiers).contains(ModifierFlags::Ambient) {
        return SubtreeContainsTypeScript;
    }
    propagate_modifier_list_subtree_facts(modifiers.as_ref())
        | propagate_subtree_facts(name)
        | propagate_eraseable_syntax_list_subtree_facts(type_parameters)
        | propagate_node_list_subtree_facts(heritage_clauses, |n| propagate_subtree_facts(Some(n)))
        | propagate_node_list_subtree_facts(Some(members), |n| propagate_subtree_facts(Some(n)))
}

fn compute_subtree_facts_class_static_block_declaration(
    d: &crate::ast::node_data_generated::ClassStaticBlockDeclarationData,
) -> SubtreeFacts {
    propagate_modifier_list_subtree_facts(d.modifiers.as_ref())
        | propagate_subtree_facts(Some(&d.body))
        | SubtreeContainsClassFields
}

fn compute_subtree_facts_constructor_declaration(
    d: &crate::ast::node_data_generated::ConstructorDeclarationData,
) -> SubtreeFacts {
    if d.body.is_none() {
        return SubtreeContainsTypeScript;
    }
    propagate_modifier_list_subtree_facts(d.modifiers.as_ref())
        | propagate_eraseable_syntax_list_subtree_facts(d.type_parameters.as_ref())
        | propagate_node_list_subtree_facts(Some(&d.parameters), |n| propagate_subtree_facts(Some(n)))
        | propagate_eraseable_syntax_subtree_facts(d.type_node.as_ref())
        | propagate_eraseable_syntax_subtree_facts(d.full_signature.as_ref())
        | propagate_subtree_facts(d.body.as_ref())
}

fn compute_subtree_facts_decorator(
    d: &crate::ast::node_data_generated::DecoratorData,
) -> SubtreeFacts {
    propagate_subtree_facts(Some(&d.expression)) | SubtreeContainsTypeScript | SubtreeContainsDecorators
}
