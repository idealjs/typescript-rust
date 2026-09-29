use crate::ast::node_flags::ModifierFlags;
use crate::ast::subtree_facts::*;
use crate::ast::utilities_misc::is_this_identifier;
use crate::ast::node_data_generated::is_identifier;
use crate::ast::syntax_kind_generated::SyntaxKind;
use crate::scanner::error_callback::TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE;

use super::m3c_2::modifier_flags_of;

pub fn compute_subtree_facts_keyword_expression(kind: SyntaxKind) -> SubtreeFacts {
    match kind {
        SyntaxKind::ThisKeyword => SubtreeContainsLexicalThis,
        SyntaxKind::SuperKeyword => SubtreeContainsLexicalSuper,
        _ => SubtreeFactsNone,
    }
}

pub fn compute_subtree_facts_meta_property(
    d: &crate::ast::node_data_generated::MetaPropertyData,
) -> SubtreeFacts {
    propagate_subtree_facts(Some(&d.name)) & !SubtreeContainsIdentifier
}

pub fn compute_subtree_facts_method_declaration(
    d: &crate::ast::node_data_generated::MethodDeclarationData,
) -> SubtreeFacts {
    if d.body.is_none() {
        return SubtreeContainsTypeScript;
    }
    let is_async = modifier_flags_of(&d.modifiers).contains(ModifierFlags::Async);
    let is_generator = d.asterisk_token.is_some();
    propagate_modifier_list_subtree_facts(d.modifiers.as_deref())
        | propagate_subtree_facts(d.asterisk_token.as_ref())
        | propagate_subtree_facts(Some(&d.name))
        | propagate_eraseable_syntax_subtree_facts(d.postfix_token.as_ref())
        | propagate_eraseable_syntax_list_subtree_facts(d.type_parameters.as_deref())
        | propagate_node_list_subtree_facts(Some(d.parameters.as_ref()), |child| {
            propagate_subtree_facts(Some(child))
        })
        | propagate_subtree_facts(d.body.as_ref())
        | propagate_eraseable_syntax_subtree_facts(d.type_node.as_ref())
        | propagate_eraseable_syntax_subtree_facts(d.full_signature.as_ref())
        | if is_async && is_generator {
            SubtreeContainsForAwaitOrAsyncGenerator
        } else {
            SubtreeFactsNone
        }
        | if is_async && !is_generator {
            SubtreeContainsAnyAwait
        } else {
            SubtreeFactsNone
        }
}

pub fn compute_subtree_facts_module_declaration(
    d: &crate::ast::node_data_generated::ModuleDeclarationData,
) -> SubtreeFacts {
    if modifier_flags_of(&d.modifiers).contains(ModifierFlags::Ambient) {
        return SubtreeContainsTypeScript;
    }
    propagate_modifier_list_subtree_facts(d.modifiers.as_deref())
        | propagate_subtree_facts(Some(&d.name))
        | propagate_subtree_facts(d.body.as_ref())
        | SubtreeContainsTypeScript
}

pub fn compute_subtree_facts_new_expression(
    d: &crate::ast::node_data_generated::NewExpressionData,
) -> SubtreeFacts {
    propagate_subtree_facts(Some(&d.expression))
        | propagate_eraseable_syntax_list_subtree_facts(d.type_arguments.as_deref())
        | propagate_node_list_subtree_facts(d.arguments.as_deref(), |child| {
            propagate_subtree_facts(Some(child))
        })
}

pub fn compute_subtree_facts_no_substitution_template_literal(
    d: &crate::ast::node_data_generated::NoSubstitutionTemplateLiteralData,
) -> SubtreeFacts {
    if d.template_flags & TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE != 0 {
        return SubtreeContainsInvalidTemplateEscape;
    }
    SubtreeFactsNone
}

pub fn compute_subtree_facts_non_null_expression(
    d: &crate::ast::node_data_generated::NonNullExpressionData,
) -> SubtreeFacts {
    propagate_subtree_facts(Some(&d.expression)) | SubtreeContainsTypeScript
}

pub fn compute_subtree_facts_parameter_declaration(
    d: &crate::ast::node_data_generated::ParameterDeclarationData,
) -> SubtreeFacts {
    if is_this_identifier(Some(&*d.name)) {
        return SubtreeContainsTypeScript;
    }
    propagate_modifier_list_subtree_facts(d.modifiers.as_deref())
        | propagate_subtree_facts(Some(&d.name))
        | propagate_eraseable_syntax_subtree_facts(d.question_token.as_ref())
        | propagate_eraseable_syntax_subtree_facts(d.type_node.as_ref())
        | propagate_subtree_facts(d.initializer.as_ref())
}

pub fn compute_subtree_facts_property_access_expression(
    d: &crate::ast::node_data_generated::PropertyAccessExpressionData,
) -> SubtreeFacts {
    let private_name = if !is_identifier(&d.name) {
        SubtreeContainsPrivateIdentifierInExpression
    } else {
        SubtreeFactsNone
    };
    propagate_subtree_facts(Some(&d.expression))
        | propagate_subtree_facts(d.question_dot_token.as_ref())
        | propagate_subtree_facts(Some(&d.name))
        | private_name
}

pub fn compute_subtree_facts_property_assignment(
    d: &crate::ast::node_data_generated::PropertyAssignmentData,
) -> SubtreeFacts {
    propagate_subtree_facts(Some(&d.name))
        | propagate_subtree_facts(Some(&d.type_node))
        | propagate_subtree_facts(Some(&d.initializer))
}

pub fn compute_subtree_facts_property_declaration(
    d: &crate::ast::node_data_generated::PropertyDeclarationData,
) -> SubtreeFacts {
    propagate_modifier_list_subtree_facts(d.modifiers.as_deref())
        | propagate_subtree_facts(Some(&d.name))
        | propagate_eraseable_syntax_subtree_facts(d.postfix_token.as_ref())
        | propagate_eraseable_syntax_subtree_facts(d.type_node.as_ref())
        | propagate_subtree_facts(d.initializer.as_ref())
        | SubtreeContainsClassFields
}

pub fn compute_subtree_facts_return_statement(
    d: &crate::ast::node_data_generated::ReturnStatementData,
) -> SubtreeFacts {
    propagate_subtree_facts(d.expression.as_ref()) | SubtreeContainsForAwaitOrAsyncGenerator
}

pub fn compute_subtree_facts_satisfies_expression(
    d: &crate::ast::node_data_generated::SatisfiesExpressionData,
) -> SubtreeFacts {
    propagate_subtree_facts(Some(&d.expression)) | SubtreeContainsTypeScript
}

pub fn compute_subtree_facts_shorthand_property_assignment(
    d: &crate::ast::node_data_generated::ShorthandPropertyAssignmentData,
) -> SubtreeFacts {
    propagate_subtree_facts(Some(&d.name))
        | propagate_subtree_facts(Some(&d.type_node))
        | propagate_subtree_facts(d.object_assignment_initializer.as_ref())
        | SubtreeContainsTypeScript
}
