use std::sync::Arc;

use crate::ast::node_data_generated::{is_array_literal_expression, is_external_module_reference, is_identifier, is_private_identifier};

use crate::ast::node::Node;
use crate::ast::node_flags::ModifierFlags;
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
use super::m3c_2::{
    propagate_eraseable_syntax_list_subtree_facts, propagate_eraseable_syntax_subtree_facts,
    propagate_modifier_list_subtree_facts, propagate_node_list_subtree_facts,
    propagate_subtree_facts,
};
use crate::ast::syntax_kind_generated::SyntaxKind;
use super::w2::contains_object_rest_or_spread;
use crate::ast::utilities_misc::is_this_identifier;
use crate::scanner::error_callback::TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE;

use super::m3c_2::modifier_flags_of;

pub fn compute_subtree_facts_enum_declaration(
    d: &crate::ast::node_data_generated::EnumDeclarationData,
) -> SubtreeFacts {
    if modifier_flags_of(&d.modifiers).contains(ModifierFlags::Ambient) {
        return SubtreeContainsTypeScript;
    }
    propagate_modifier_list_subtree_facts(d.modifiers.as_ref())
        | propagate_subtree_facts(Some(&d.name))
        | propagate_node_list_subtree_facts(Some(&d.members), |n| propagate_subtree_facts(Some(n)))
        | SubtreeContainsTypeScript
}

pub fn compute_subtree_facts_enum_member(
    d: &crate::ast::node_data_generated::EnumMemberData,
) -> SubtreeFacts {
    propagate_subtree_facts(Some(&d.name))
        | propagate_subtree_facts(d.initializer.as_ref())
        | SubtreeContainsTypeScript
}

pub fn compute_subtree_facts_export_assignment(
    d: &crate::ast::node_data_generated::ExportAssignmentData,
) -> SubtreeFacts {
    propagate_modifier_list_subtree_facts(d.modifiers.as_ref())
        | propagate_subtree_facts(Some(&d.type_node))
        | propagate_subtree_facts(Some(&d.expression))
        | if d.is_export_equals {
            SubtreeContainsTypeScript
        } else {
            SubtreeFactsNone
        }
}

pub fn compute_subtree_facts_export_declaration(
    d: &crate::ast::node_data_generated::ExportDeclarationData,
) -> SubtreeFacts {
    propagate_modifier_list_subtree_facts(d.modifiers.as_ref())
        | propagate_subtree_facts(d.export_clause.as_ref())
        | propagate_subtree_facts(d.module_specifier.as_ref())
        | propagate_subtree_facts(d.attributes.as_ref())
        | if d.is_type_only {
            SubtreeContainsTypeScript
        } else {
            SubtreeFactsNone
        }
}

pub fn compute_subtree_facts_export_specifier(
    d: &crate::ast::node_data_generated::ExportSpecifierData,
) -> SubtreeFacts {
    if d.is_type_only {
        return SubtreeContainsTypeScript;
    }
    propagate_subtree_facts(d.property_name.as_ref()) | propagate_subtree_facts(Some(&d.name))
}

pub fn compute_subtree_facts_expression_with_type_arguments(
    d: &crate::ast::node_data_generated::ExpressionWithTypeArgumentsData,
) -> SubtreeFacts {
    propagate_subtree_facts(Some(&d.expression))
        | propagate_eraseable_syntax_list_subtree_facts(d.type_arguments.as_ref())
}

pub fn compute_subtree_facts_for_in_of_statement(
    d: &crate::ast::node_data_generated::ForInOrOfStatementData,
) -> SubtreeFacts {
    propagate_subtree_facts(Some(&d.initializer))
        | propagate_subtree_facts(Some(&d.expression))
        | propagate_subtree_facts(Some(&d.statement))
        | if d.await_modifier.is_some() {
            SubtreeContainsForAwaitOrAsyncGenerator
        } else {
            SubtreeFactsNone
        }
}

pub fn compute_subtree_facts_function_declaration(
    d: &crate::ast::node_data_generated::FunctionDeclarationData,
) -> SubtreeFacts {
    if d.body.is_none() || modifier_flags_of(&d.modifiers).contains(ModifierFlags::Ambient) {
        return SubtreeContainsTypeScript;
    }
    let is_async = modifier_flags_of(&d.modifiers).contains(ModifierFlags::Async);
    let is_generator = d.asterisk_token.is_some();
    propagate_modifier_list_subtree_facts(d.modifiers.as_ref())
        | propagate_subtree_facts(d.asterisk_token.as_ref())
        | propagate_subtree_facts(d.name.as_ref())
        | propagate_eraseable_syntax_list_subtree_facts(d.type_parameters.as_ref())
        | propagate_node_list_subtree_facts(Some(&d.parameters), |n| propagate_subtree_facts(Some(n)))
        | propagate_eraseable_syntax_subtree_facts(d.type_node.as_ref())
        | propagate_eraseable_syntax_subtree_facts(d.full_signature.as_ref())
        | propagate_subtree_facts(d.body.as_ref())
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

pub fn compute_subtree_facts_function_expression(
    d: &crate::ast::node_data_generated::FunctionExpressionData,
) -> SubtreeFacts {
    let is_async = modifier_flags_of(&d.modifiers).contains(ModifierFlags::Async);
    let is_generator = d.asterisk_token.is_some();
    propagate_modifier_list_subtree_facts(d.modifiers.as_ref())
        | propagate_subtree_facts(d.asterisk_token.as_ref())
        | propagate_subtree_facts(d.name.as_ref())
        | propagate_eraseable_syntax_list_subtree_facts(d.type_parameters.as_ref())
        | propagate_node_list_subtree_facts(Some(&d.parameters), |n| propagate_subtree_facts(Some(n)))
        | propagate_eraseable_syntax_subtree_facts(d.type_node.as_ref())
        | propagate_eraseable_syntax_subtree_facts(d.full_signature.as_ref())
        | propagate_subtree_facts(Some(&d.body))
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

pub fn compute_subtree_facts_heritage_clause(
    d: &crate::ast::node_data_generated::HeritageClauseData,
) -> SubtreeFacts {
    match d.token {
        SyntaxKind::ExtendsKeyword => {
            propagate_node_list_subtree_facts(Some(&d.types), |n| propagate_subtree_facts(Some(n)))
        }
        SyntaxKind::ImplementsKeyword => SubtreeContainsTypeScript,
        _ => SubtreeFactsNone,
    }
}

pub fn compute_subtree_facts_import_clause(
    d: &crate::ast::node_data_generated::ImportClauseData,
) -> SubtreeFacts {
    if d.phase_modifier == Some(SyntaxKind::TypeKeyword) {
        return SubtreeContainsTypeScript;
    }
    propagate_subtree_facts(d.name.as_ref()) | propagate_subtree_facts(d.named_bindings.as_ref())
}

pub fn compute_subtree_facts_import_equals_declaration(
    d: &crate::ast::node_data_generated::ImportEqualsDeclarationData,
) -> SubtreeFacts {
    if d.is_type_only || !is_external_module_reference(&d.module_reference) {
        return SubtreeContainsTypeScript;
    }
    propagate_modifier_list_subtree_facts(d.modifiers.as_ref())
        | propagate_subtree_facts(Some(&d.name))
        | propagate_subtree_facts(Some(&d.module_reference))
}

pub fn compute_subtree_facts_import_specifier(
    d: &crate::ast::node_data_generated::ImportSpecifierData,
) -> SubtreeFacts {
    if d.is_type_only {
        return SubtreeContainsTypeScript;
    }
    propagate_subtree_facts(d.property_name.as_ref()) | propagate_subtree_facts(Some(&d.name))
}

pub fn compute_subtree_facts_jsx_attribute(
    d: &crate::ast::node_data_generated::JsxAttributeData,
) -> SubtreeFacts {
    propagate_subtree_facts(Some(&d.name))
        | propagate_subtree_facts(d.initializer.as_ref())
        | SubtreeContainsJsx
}

pub fn compute_subtree_facts_jsx_attributes(
    d: &crate::ast::node_data_generated::JsxAttributesData,
) -> SubtreeFacts {
    propagate_node_list_subtree_facts(Some(&d.properties), |n| propagate_subtree_facts(Some(n)))
        | SubtreeContainsJsx
}

pub fn compute_subtree_facts_jsx_closing_element(
    d: &crate::ast::node_data_generated::JsxClosingElementData,
) -> SubtreeFacts {
    propagate_subtree_facts(Some(&d.tag_name)) | SubtreeContainsJsx
}

pub fn compute_subtree_facts_jsx_element(
    d: &crate::ast::node_data_generated::JsxElementData,
) -> SubtreeFacts {
    propagate_subtree_facts(Some(&d.opening_element))
        | propagate_node_list_subtree_facts(Some(&d.children), |n| propagate_subtree_facts(Some(n)))
        | propagate_subtree_facts(Some(&d.closing_element))
        | SubtreeContainsJsx
}

pub fn compute_subtree_facts_jsx_expression(
    d: &crate::ast::node_data_generated::JsxExpressionData,
) -> SubtreeFacts {
    propagate_subtree_facts(d.expression.as_ref()) | SubtreeContainsJsx
}

pub fn compute_subtree_facts_jsx_fragment(
    d: &crate::ast::node_data_generated::JsxFragmentData,
) -> SubtreeFacts {
    propagate_node_list_subtree_facts(Some(&d.children), |n| propagate_subtree_facts(Some(n)))
        | SubtreeContainsJsx
}

pub fn compute_subtree_facts_jsx_namespaced_name(
    d: &crate::ast::node_data_generated::JsxNamespacedNameData,
) -> SubtreeFacts {
    propagate_subtree_facts(Some(&d.namespace))
        | propagate_subtree_facts(Some(&d.name))
        | SubtreeContainsJsx
}

pub fn compute_subtree_facts_jsx_opening_element(
    d: &crate::ast::node_data_generated::JsxOpeningElementData,
) -> SubtreeFacts {
    propagate_subtree_facts(Some(&d.tag_name))
        | propagate_eraseable_syntax_list_subtree_facts(d.type_arguments.as_ref())
        | propagate_subtree_facts(Some(&d.attributes))
        | SubtreeContainsJsx
}

pub fn compute_subtree_facts_jsx_self_closing_element(
    d: &crate::ast::node_data_generated::JsxSelfClosingElementData,
) -> SubtreeFacts {
    propagate_subtree_facts(Some(&d.tag_name))
        | propagate_eraseable_syntax_list_subtree_facts(d.type_arguments.as_ref())
        | propagate_subtree_facts(Some(&d.attributes))
        | SubtreeContainsJsx
}

pub fn compute_subtree_facts_jsx_spread_attribute(
    d: &crate::ast::node_data_generated::JsxSpreadAttributeData,
) -> SubtreeFacts {
    propagate_subtree_facts(Some(&d.expression)) | SubtreeContainsJsx
}

