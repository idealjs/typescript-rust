use crate::ast::*;
use std::sync::Arc;
use tsox_core::core::compiler_options::{CompilerOptions, ModuleKind};
use tsox_core::tspath::Path as TsPath;
use super::m3b_2::NodeFactory;
use super::m3f_2::{has_abstract_modifier, has_ambient_modifier, node_parameters};
use super::m3g_2::{
    is_part_of_type_query, is_this_parameter, is_type_only_import_or_export_declaration,
};
use super::m3h::{
    is_identifier_in_non_emitting_heritage_clause,
    is_part_of_possibly_valid_type_or_abstract_computed_property_name,
};
use super::m3h_2::is_shorthand_property_name_use_site;
use super::w5::get_leftmost_access_expression;
use super::w7a::is_jsdoc_type_assertion;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct OuterExpressionKinds: u32 {
        const PARENS = 1 << 0;
        const TYPE_ASSERTIONS = 1 << 1;
        const EXPRESSIONS_WITH_TYPE_ARGUMENTS = 1 << 2;
        const NON_NULL_ASSERTIONS = 1 << 3;
        const PARTIALLY_EMITTED_EXPRESSIONS = 1 << 4;
        const ASSIGNMENTS = 1 << 5;
        const COMMA = 1 << 6;
        const SATISFIES = 1 << 7;
        const EXCLUDE_JSDOC_TYPE_ASSERTION = 1 << 8;
    }
}

impl OuterExpressionKinds {
    pub const ALL: OuterExpressionKinds = OuterExpressionKinds::from_bits_truncate(
        OuterExpressionKinds::PARENS.bits()
            | OuterExpressionKinds::TYPE_ASSERTIONS.bits()
            | OuterExpressionKinds::EXPRESSIONS_WITH_TYPE_ARGUMENTS.bits()
            | OuterExpressionKinds::NON_NULL_ASSERTIONS.bits()
            | OuterExpressionKinds::PARTIALLY_EMITTED_EXPRESSIONS.bits()
            | OuterExpressionKinds::ASSIGNMENTS.bits()
            | OuterExpressionKinds::COMMA.bits()
            | OuterExpressionKinds::SATISFIES.bits(),
    );
}

pub fn is_outer_expression(node: &Node, kinds: OuterExpressionKinds) -> bool { ::tsox_core::fntrace::enter("is_outer_expression"); 
    match node.kind {
        SyntaxKind::ParenthesizedExpression => {
            kinds.intersects(OuterExpressionKinds::PARENS)
                && !(kinds.intersects(OuterExpressionKinds::EXCLUDE_JSDOC_TYPE_ASSERTION)
                    && is_jsdoc_type_assertion(Some(node)))
        }
        SyntaxKind::TypeAssertionExpression | SyntaxKind::AsExpression => {
            kinds.intersects(OuterExpressionKinds::TYPE_ASSERTIONS)
        }
        SyntaxKind::SatisfiesExpression => {
            kinds.intersects(OuterExpressionKinds::EXPRESSIONS_WITH_TYPE_ARGUMENTS.union(OuterExpressionKinds::SATISFIES))
        }
        SyntaxKind::ExpressionWithTypeArguments => {
            kinds.intersects(OuterExpressionKinds::EXPRESSIONS_WITH_TYPE_ARGUMENTS)
        }
        SyntaxKind::NonNullExpression => kinds.intersects(OuterExpressionKinds::NON_NULL_ASSERTIONS),
        SyntaxKind::PartiallyEmittedExpression => {
            kinds.intersects(OuterExpressionKinds::PARTIALLY_EMITTED_EXPRESSIONS)
        }
        SyntaxKind::BinaryExpression => match &node.data {
            NodeData::BinaryExpression(d) => match d.operator_token.kind {
                SyntaxKind::EqualsToken => kinds.intersects(OuterExpressionKinds::ASSIGNMENTS),
                SyntaxKind::CommaToken => kinds.intersects(OuterExpressionKinds::COMMA),
                _ => false,
            },
            _ => false,
        },
        _ => false,
    }
}

pub fn skip_outer_expressions(node: &Arc<Node>, kinds: OuterExpressionKinds) -> Arc<Node> { ::tsox_core::fntrace::enter("skip_outer_expressions"); 
    let mut node = node.clone();
    while is_outer_expression(&node, kinds) {
        if is_binary_expression(&node) {
            let NodeData::BinaryExpression(d) = &node.data else {
                break;
            };
            node = d.right.clone();
        } else {
            match node.expression() {
                Some(next) => node = next.clone(),
                None => break,
            }
        }
    }
    node
}

pub fn skip_parentheses(node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("skip_parentheses"); 
    skip_outer_expressions(node, OuterExpressionKinds::PARENS)
}

pub fn skip_type_parentheses(node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("skip_type_parentheses"); 
    let mut node = node.clone();
    while is_parenthesized_type_node(&node) {
        match node.type_node() {
            Some(next) => node = next.clone(),
            None => break,
        }
    }
    node
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindAncestorResult {
    True,
    False,
    Quit,
}

pub fn to_find_ancestor_result(b: bool) -> FindAncestorResult { ::tsox_core::fntrace::enter("to_find_ancestor_result"); 
    if b {
        FindAncestorResult::True
    } else {
        FindAncestorResult::False
    }
}

pub fn node_kind_is(node: &Node, kinds: &[SyntaxKind]) -> bool { ::tsox_core::fntrace::enter("node_kind_is"); 
    kinds.contains(&node.kind)
}

pub fn node_has_kind(node: Option<&Node>, kind: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("node_has_kind"); 
    match node {
        Some(node) => node.kind == kind,
        None => false,
    }
}

use crate::scanner::TOKEN_FLAGS_UNTERMINATED;

pub fn is_unterminated_literal(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_unterminated_literal"); 
    match &node.data {
        NodeData::StringLiteral(d) => d.token_flags & TOKEN_FLAGS_UNTERMINATED != 0,
        NodeData::NumericLiteral(d) => d.token_flags & TOKEN_FLAGS_UNTERMINATED != 0,
        NodeData::BigIntLiteral(d) => d.token_flags & TOKEN_FLAGS_UNTERMINATED != 0,
        NodeData::NoSubstitutionTemplateLiteral(d) => d.template_flags & TOKEN_FLAGS_UNTERMINATED != 0,
        NodeData::TemplateHead(d) => d.template_flags & TOKEN_FLAGS_UNTERMINATED != 0,
        NodeData::TemplateMiddle(d) => d.template_flags & TOKEN_FLAGS_UNTERMINATED != 0,
        NodeData::TemplateTail(d) => d.template_flags & TOKEN_FLAGS_UNTERMINATED != 0,
        _ => false,
    }
}

fn is_in_expression_context(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_in_expression_context");
    let Some(parent) = node.parent() else {
        return false;
    };
    let same = |n: &Option<Arc<Node>>| n.as_ref().is_some_and(|n| std::ptr::eq(n.as_ref(), node));
    match parent.kind {
        SyntaxKind::VariableDeclaration
        | SyntaxKind::Parameter
        | SyntaxKind::PropertyDeclaration
        | SyntaxKind::PropertySignature
        | SyntaxKind::EnumMember
        | SyntaxKind::PropertyAssignment
        | SyntaxKind::BindingElement => match &parent.data {
            NodeData::VariableDeclaration(d) => same(&d.initializer),
            NodeData::ParameterDeclaration(d) => same(&d.initializer),
            NodeData::PropertyDeclaration(d) => same(&d.initializer),
            NodeData::PropertySignatureDeclaration(d) => std::ptr::eq(d.initializer.as_ref(), node),
            NodeData::EnumMember(d) => same(&d.initializer),
            NodeData::PropertyAssignment(d) => std::ptr::eq(d.initializer.as_ref(), node),
            NodeData::BindingElement(d) => same(&d.initializer),
            _ => false,
        },
        SyntaxKind::ExpressionStatement
        | SyntaxKind::IfStatement
        | SyntaxKind::DoStatement
        | SyntaxKind::WhileStatement
        | SyntaxKind::ReturnStatement
        | SyntaxKind::WithStatement
        | SyntaxKind::SwitchStatement
        | SyntaxKind::CaseClause
        | SyntaxKind::DefaultClause
        | SyntaxKind::ThrowStatement
        | SyntaxKind::TypeAssertionExpression
        | SyntaxKind::AsExpression
        | SyntaxKind::TemplateSpan
        | SyntaxKind::ComputedPropertyName
        | SyntaxKind::SatisfiesExpression => parent
            .expression()
            .is_some_and(|e| std::ptr::eq(e.as_ref(), node)),
        SyntaxKind::ForStatement => match &parent.data {
            NodeData::ForStatement(s) => {
                (same(&s.initializer)
                    && s
                        .initializer
                        .as_ref()
                        .is_some_and(|i| i.kind != SyntaxKind::VariableDeclarationList))
                    || s
                        .condition
                        .as_ref()
                        .is_some_and(|c| std::ptr::eq(c.as_ref(), node))
                    || s
                        .incrementor
                        .as_ref()
                        .is_some_and(|i| std::ptr::eq(i.as_ref(), node))
            }
            _ => false,
        },
        SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement => match &parent.data {
            NodeData::ForInOrOfStatement(s) => {
                (std::ptr::eq(s.initializer.as_ref(), node)
                    && s.initializer.kind != SyntaxKind::VariableDeclarationList)
                    || std::ptr::eq(s.expression.as_ref(), node)
            }
            _ => false,
        },
        SyntaxKind::Decorator
        | SyntaxKind::JsxExpression
        | SyntaxKind::JsxSpreadAttribute
        | SyntaxKind::SpreadAssignment => true,
        SyntaxKind::ExpressionWithTypeArguments => matches!(
            &parent.data,
            NodeData::ExpressionWithTypeArguments(d)
                if std::ptr::eq(d.expression.as_ref(), node)
        ) && !is_part_of_type_node(&parent),
        SyntaxKind::ShorthandPropertyAssignment => matches!(
            &parent.data,
            NodeData::ShorthandPropertyAssignment(d)
                if d.object_assignment_initializer
                    .as_ref()
                    .is_some_and(|i| std::ptr::eq(i.as_ref(), node))
        ),
        _ => is_expression_node(&parent),
    }
}

pub fn is_valid_type_only_alias_use_site(use_site: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_valid_type_only_alias_use_site");
    use_site.flags.intersects(NodeFlags::Ambient | NodeFlags::JSDoc)
        || is_part_of_type_query(use_site)
        || is_identifier_in_non_emitting_heritage_clause(use_site)
        || is_part_of_possibly_valid_type_or_abstract_computed_property_name(use_site)
        || !(is_expression_node(use_site) || is_shorthand_property_name_use_site(use_site))
}

pub fn is_var_await_using(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_var_await_using"); 
    get_combined_node_flags(node).intersection(NodeFlags::BlockScoped) == NodeFlags::AwaitUsing
}

pub fn is_var_const(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_var_const"); 
    get_combined_node_flags(node).intersection(NodeFlags::BlockScoped) == NodeFlags::Const
}

pub fn is_var_const_like(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_var_const_like"); 
    let scoped = get_combined_node_flags(node).intersection(NodeFlags::BlockScoped);
    matches!(
        scoped,
        NodeFlags::Const | NodeFlags::Using | NodeFlags::AwaitUsing
    )
}

pub fn is_var_let(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_var_let"); 
    get_combined_node_flags(node).intersection(NodeFlags::BlockScoped) == NodeFlags::Let
}

pub fn is_var_using(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_var_using"); 
    get_combined_node_flags(node).intersection(NodeFlags::BlockScoped) == NodeFlags::Using
}

fn is_variable_declaration_initialized_with_require_helper(
    node: &Arc<Node>,
    allow_accessed_require: bool,
) -> bool { ::tsox_core::fntrace::enter("is_variable_declaration_initialized_with_require_helper"); 
    if !is_in_js_file(node) {
        return false;
    }
    if node.kind != SyntaxKind::VariableDeclaration {
        return false;
    }
    let mut initializer = match &node.data {
        NodeData::VariableDeclaration(d) => d.initializer.clone(),
        _ => None,
    };
    if allow_accessed_require {
        if let Some(init) = &initializer {
            initializer = Some(get_leftmost_access_expression(init));
        }
    }
    let Some(initializer) = initializer else {
        return false;
    };
    let Some(statement) = node
        .parent()
        .and_then(|p| p.parent()) else {
        return false;
    };
    node_modifier_flags(&statement).intersection(ModifierFlags::Export)
        == ModifierFlags::empty()
        && is_require_call(&initializer, true)
}

pub fn is_variable_declaration_initialized_to_require(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_variable_declaration_initialized_to_require"); 
    let mut node = node.clone();
    if node.kind == SyntaxKind::BindingElement {
        match node.parent().and_then(|p| p.parent()) {
            Some(up) => node = up,
            None => return false,
        }
    }
    is_variable_declaration_initialized_with_require_helper(&node, false)
}

pub fn is_variable_declaration_initialized_to_bare_or_accessed_require(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_variable_declaration_initialized_to_bare_or_accessed_require"); 
    is_variable_declaration_initialized_with_require_helper(node, true)
}

pub fn is_variable_like(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_variable_like"); 
    matches!(
        node.kind,
        SyntaxKind::BindingElement
            | SyntaxKind::EnumMember
            | SyntaxKind::Parameter
            | SyntaxKind::PropertyAssignment
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature
            | SyntaxKind::ShorthandPropertyAssignment
            | SyntaxKind::VariableDeclaration
    )
}

pub fn is_variable_parameter_or_property(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_variable_parameter_or_property"); 
    matches!(
        node.kind,
        SyntaxKind::VariableDeclaration
            | SyntaxKind::Parameter
            | SyntaxKind::PropertySignature
            | SyntaxKind::PropertyDeclaration
    )
}

pub fn module_export_name_is_default(node: &Node) -> bool { ::tsox_core::fntrace::enter("module_export_name_is_default"); 
    node.text() == INTERNAL_SYMBOL_NAME_DEFAULT
}

pub struct HasFileNameImpl {
    file_name: String,
    path: TsPath,
}

pub fn new_has_file_name(file_name: String, path: TsPath) -> HasFileNameImpl { ::tsox_core::fntrace::enter("new_has_file_name"); 
    HasFileNameImpl { file_name, path }
}

impl HasFileNameImpl {
    pub fn file_name(&self) -> &str { ::tsox_core::fntrace::enter("file_name"); 
        &self.file_name
    }

    pub fn path(&self) -> &TsPath { ::tsox_core::fntrace::enter("path"); 
        &self.path
    }
}

pub fn node_can_be_decorated(
    use_legacy_decorators: bool,
    node: &Arc<Node>,
    parent: Option<&Arc<Node>>,
    grandparent: Option<&Arc<Node>>,
) -> bool { ::tsox_core::fntrace::enter("node_can_be_decorated"); 
    if use_legacy_decorators
        && node.name().is_some_and(|n| is_private_identifier(n))
    {
        return false;
    }
    match node.kind {
        SyntaxKind::ClassDeclaration => true,
        SyntaxKind::ClassExpression => !use_legacy_decorators,
        SyntaxKind::PropertyDeclaration => match parent {
            Some(parent) => {
                (use_legacy_decorators && is_class_declaration(parent))
                    || (!use_legacy_decorators
                        && is_class_like(parent)
                        && !has_abstract_modifier(node)
                        && !has_ambient_modifier(node))
            }
            None => false,
        },
        SyntaxKind::GetAccessor
        | SyntaxKind::SetAccessor
        | SyntaxKind::MethodDeclaration => {
            match parent {
                Some(parent) => {
                    node_body(node).is_some()
                        && ((use_legacy_decorators && is_class_declaration(parent))
                            || (!use_legacy_decorators && is_class_like(parent)))
                }
                None => false,
            }
        }
        SyntaxKind::Parameter => {
            if !use_legacy_decorators {
                return false;
            }
            match parent {
                Some(parent) => {
                    node_body(parent).is_some()
                        && matches!(
                            parent.kind,
                            SyntaxKind::Constructor
                                | SyntaxKind::MethodDeclaration
                                | SyntaxKind::SetAccessor
                        )
                        && !get_this_parameter(parent)
                            .map(|tp| Arc::ptr_eq(&tp, node))
                            .unwrap_or(false)
                        && grandparent.is_some_and(|g| g.kind == SyntaxKind::ClassDeclaration)
                }
                None => false,
            }
        }
        _ => false,
    }
}

pub fn node_is_decorated(
    use_legacy_decorators: bool,
    node: &Arc<Node>,
    parent: Option<&Arc<Node>>,
    grandparent: Option<&Arc<Node>>,
) -> bool { ::tsox_core::fntrace::enter("node_is_decorated"); 
    has_decorators(node)
        && node_can_be_decorated(use_legacy_decorators, node, parent, grandparent)
}

pub fn node_or_child_is_decorated(
    use_legacy_decorators: bool,
    node: &Arc<Node>,
    parent: Option<&Arc<Node>>,
    grandparent: Option<&Arc<Node>>,
) -> bool { ::tsox_core::fntrace::enter("node_or_child_is_decorated"); 
    node_is_decorated(use_legacy_decorators, node, parent, grandparent)
        || child_is_decorated(use_legacy_decorators, node, parent)
}

fn child_is_decorated(
    use_legacy_decorators: bool,
    node: &Arc<Node>,
    parent: Option<&Arc<Node>>,
) -> bool { ::tsox_core::fntrace::enter("child_is_decorated"); 
    match node.kind {
        SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => {
            node_members(node).iter().any(|m| {
                node_or_child_is_decorated(use_legacy_decorators, m, Some(node), parent)
            })
        }
        SyntaxKind::MethodDeclaration
        | SyntaxKind::SetAccessor
        | SyntaxKind::Constructor => node_parameters(node).map_or(false, |parameters| {
            parameters.nodes.iter().any(|p| {
                node_is_decorated(use_legacy_decorators, p, Some(node), parent)
            })
        }),
        _ => false,
    }
}

pub fn should_transform_import_call(
    file_name: &str,
    options: &CompilerOptions,
    implied_node_format_for_emit: ModuleKind,
) -> bool { ::tsox_core::fntrace::enter("should_transform_import_call"); 
    let module_kind = options.get_emit_module_kind();
    if (ModuleKind::Node16 <= module_kind && module_kind <= ModuleKind::NodeNext)
        || module_kind == ModuleKind::Preserve
    {
        return false;
    }
    implied_node_format_for_emit < ModuleKind::ES2015
}

pub fn tag_names_are_equivalent(lhs: &Node, rhs: &Node) -> bool { ::tsox_core::fntrace::enter("tag_names_are_equivalent"); 
    if lhs.kind != rhs.kind {
        return false;
    }
    match lhs.kind {
        SyntaxKind::Identifier => lhs.text() == rhs.text(),
        SyntaxKind::ThisKeyword => true,
        SyntaxKind::JsxNamespacedName => match (&lhs.data, &rhs.data) {
            (
                NodeData::JsxNamespacedName(l),
                NodeData::JsxNamespacedName(r),
            ) => l.namespace.text() == r.namespace.text() && lhs.name().zip(rhs.name()).is_some_and(|(ln, rn)| ln.text() == rn.text()),
            _ => false,
        },
        SyntaxKind::PropertyAccessExpression => match (&lhs.data, &rhs.data) {
            (
                NodeData::PropertyAccessExpression(l),
                NodeData::PropertyAccessExpression(r),
            ) => {
                l.name.text() == r.name.text()
                    && lhs
                        .expression()
                        .zip(rhs.expression())
                        .is_some_and(|(le, re)| tag_names_are_equivalent(le, re))
            }
            _ => false,
        },
        _ => false,
    }
}

pub fn try_get_ambient_module_name_from_symbol_name(s: &str) -> Option<String> { ::tsox_core::fntrace::enter("try_get_ambient_module_name_from_symbol_name"); 
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        return Some(s[1..s.len() - 1].to_string());
    }
    let pattern_prefix = format!("{}\"", INTERNAL_SYMBOL_NAME_PREFIX);
    let Some(rest) = s.strip_prefix(&pattern_prefix) else {
        return None;
    };
    let marker = "\"pattern@";
    let marker_index = rest.rfind(marker);
    if marker_index.is_none() || marker_index.unwrap() < 1 {
        return None;
    }
    Some(rest[..marker_index.unwrap()].to_string())
}

pub fn try_get_class_extending_expression_with_type_arguments(
    node: &Arc<Node>,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("try_get_class_extending_expression_with_type_arguments"); 
    if !is_expression_with_type_arguments(node) {
        return None;
    }
    match try_get_class_implementing_or_extending_heritage_clause_element(node) {
        Some((class, is_implements)) if !is_implements => Some(class),
        _ => None,
    }
}

pub fn try_get_class_implementing_or_extending_heritage_clause_element(
    node: &Arc<Node>,
) -> Option<(Arc<Node>, bool)> { ::tsox_core::fntrace::enter("try_get_class_implementing_or_extending_heritage_clause_element"); 
    let parent = node.parent()?;
    if !(is_expression_with_type_arguments(node) || is_type_reference_node(node)) {
        return None;
    }
    if !is_heritage_clause(&parent) {
        return None;
    }
    let grandparent = parent.parent()?;
    if !is_class_like(&grandparent) {
        return None;
    }
    let is_implements = match &parent.data {
        NodeData::HeritageClause(d) => d.token == SyntaxKind::ImplementsKeyword,
        _ => false,
    };
    Some((grandparent, is_implements))
}

pub fn try_get_import_from_module_specifier(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("try_get_import_from_module_specifier"); 
    let parent = node.parent()?;
    match parent.kind {
        SyntaxKind::ImportDeclaration
        | SyntaxKind::JSImportDeclaration
        | SyntaxKind::ExportDeclaration => Some(parent),
        SyntaxKind::ExternalModuleReference => parent.parent(),
        SyntaxKind::CallExpression => {
            if is_import_call(&parent) || is_require_call(&parent, false) {
                Some(parent)
            } else {
                None
            }
        }
        SyntaxKind::LiteralType => {
            if !is_string_literal(node) {
                return None;
            }
            let grandparent = parent.parent()?;
            if is_import_type_node(&grandparent) {
                Some(grandparent)
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn try_get_property_name_of_binding_or_assignment_element(
    binding_element: &Node,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("try_get_property_name_of_binding_or_assignment_element"); 
    match binding_element.kind {
        SyntaxKind::BindingElement => match &binding_element.data {
            NodeData::BindingElement(d) => {
                let property_name = d.property_name.as_ref()?;
                try_get_property_name_of_property_name(property_name)
            }
            _ => None,
        },
        SyntaxKind::PropertyAssignment => match &binding_element.data {
            NodeData::PropertyAssignment(d) => {
                try_get_property_name_of_property_name(&d.name)
            }
            _ => None,
        },
        SyntaxKind::SpreadAssignment => binding_element.name().cloned(),
        _ => None,
    }
}

fn try_get_property_name_of_property_name(property_name: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("try_get_property_name_of_property_name"); 
    match &property_name.data {
        NodeData::ComputedPropertyName(d)
            if is_string_or_numeric_literal_like(&d.expression) =>
        {
            Some(d.expression.clone())
        }
        _ => Some(property_name.clone()),
    }
}

pub fn set_parent_in_children(node: &Arc<Node>) { ::tsox_core::fntrace::enter("set_parent_in_children"); 
    for_each_child(node, &mut |child: &Arc<Node>| {
        child.set_parent(node);
        set_parent_in_children(child);
        false
    });
}

pub fn set_imports_of_source_file(node: &mut SourceFile, imports: Vec<Arc<Node>>) { ::tsox_core::fntrace::enter("set_imports_of_source_file"); 
    node.imports = imports;
}

fn is_part_of_type_expression_with_type_arguments(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_part_of_type_expression_with_type_arguments"); 
    let Some(parent) = node.parent() else {
        return false;
    };
    let Some(grandparent) = parent.parent() else {
        return false;
    };
    let heritage = is_heritage_clause(&parent)
        && (!is_class_like(&grandparent)
            || matches!(&parent.data, NodeData::HeritageClause(d) if d.token == SyntaxKind::ImplementsKeyword));
    heritage || is_jsdoc_implements_tag(&parent) || is_jsdoc_augments_tag(&parent)
}

fn is_part_of_type_node_in_parent(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_part_of_type_node_in_parent"); 
    let Some(parent) = node.parent() else {
        return false;
    };
    if parent.kind == SyntaxKind::TypeQuery {
        return false;
    }
    if parent.kind == SyntaxKind::ImportType {
        return match &parent.data {
            NodeData::ImportTypeNode(d) => !d.is_type_of,
            _ => false,
        };
    }
    if (parent.kind as i16) >= (SyntaxKind::TypeReference as i16)
        && (parent.kind as i16) <= (SyntaxKind::ImportType as i16)
    {
        return true;
    }
    match parent.kind {
        SyntaxKind::ExpressionWithTypeArguments => is_part_of_type_expression_with_type_arguments(&parent),
        SyntaxKind::TypeParameter => match (&parent.data, node.name().is_none()) {
            (NodeData::TypeParameterDeclaration(d), true) => {
                d.constraint.as_ref().is_some_and(|c| Arc::ptr_eq(c, node))
            }
            _ => false,
        },
        SyntaxKind::VariableDeclaration
        | SyntaxKind::Parameter
        | SyntaxKind::PropertyDeclaration
        | SyntaxKind::PropertySignature
        | SyntaxKind::FunctionDeclaration
        | SyntaxKind::FunctionExpression
        | SyntaxKind::ArrowFunction
        | SyntaxKind::Constructor
        | SyntaxKind::MethodDeclaration
        | SyntaxKind::MethodSignature
        | SyntaxKind::GetAccessor
        | SyntaxKind::SetAccessor
        | SyntaxKind::CallSignature
        | SyntaxKind::ConstructSignature
        | SyntaxKind::IndexSignature
        | SyntaxKind::TypeAssertionExpression
        | SyntaxKind::ReturnStatement
        | SyntaxKind::CallExpression
        | SyntaxKind::NewExpression => match parent.type_node() {
            Some(type_node) => Arc::ptr_eq(type_node, node),
            None => false,
        },
        _ => false,
    }
}

pub fn is_part_of_type_node(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_part_of_type_node"); 
    let kind = node.kind;
    if (kind as i16) >= (SyntaxKind::TypeReference as i16)
        && (kind as i16) <= (SyntaxKind::ImportType as i16)
    {
        return true;
    }
    match kind {
        SyntaxKind::AnyKeyword
        | SyntaxKind::UnknownKeyword
        | SyntaxKind::NumberKeyword
        | SyntaxKind::BigIntKeyword
        | SyntaxKind::StringKeyword
        | SyntaxKind::BooleanKeyword
        | SyntaxKind::SymbolKeyword
        | SyntaxKind::ObjectKeyword
        | SyntaxKind::UndefinedKeyword
        | SyntaxKind::NullKeyword
        | SyntaxKind::NeverKeyword => true,
        SyntaxKind::VoidKeyword => node
            .parent()
            .is_some_and(|p| p.kind != SyntaxKind::VoidExpression),
        SyntaxKind::ExpressionWithTypeArguments => is_part_of_type_expression_with_type_arguments(node),
        SyntaxKind::TypeParameter => node
            .parent()
            .is_some_and(|p| matches!(p.kind, SyntaxKind::MappedType | SyntaxKind::InferType)),
        SyntaxKind::Identifier => {
            let Some(parent) = node.parent() else {
                return false;
            };
            match &parent.data {
                NodeData::QualifiedName(d) if Arc::ptr_eq(&d.right, node) => {
                    is_part_of_type_node_in_parent(&parent)
                }
                NodeData::PropertyAccessExpression(d) if Arc::ptr_eq(&d.name, node) => {
                    is_part_of_type_node_in_parent(&parent)
                }
                _ => is_part_of_type_node_in_parent(node),
            }
        }
        SyntaxKind::QualifiedName
        | SyntaxKind::PropertyAccessExpression
        | SyntaxKind::ThisKeyword => is_part_of_type_node_in_parent(node),
        _ => false,
    }
}

pub fn is_part_of_type_only_import_or_export_declaration(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_part_of_type_only_import_or_export_declaration"); 
    find_ancestor(node, |ancestor| {
        is_type_only_import_or_export_declaration(ancestor)
    })
    .is_some()
}

fn node_modifier_flags(node: &Node) -> ModifierFlags { ::tsox_core::fntrace::enter("node_modifier_flags"); 
    node.syntactic_modifier_flags()
}

fn has_decorators(node: &Node) -> bool { ::tsox_core::fntrace::enter("has_decorators"); 
    has_syntactic_modifier(node, ModifierFlags::Decorator)
}

pub fn is_expression_node(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_expression_node"); 
    match node.kind {
        SyntaxKind::SuperKeyword
        | SyntaxKind::NullKeyword
        | SyntaxKind::TrueKeyword
        | SyntaxKind::FalseKeyword
        | SyntaxKind::RegularExpressionLiteral
        | SyntaxKind::ArrayLiteralExpression
        | SyntaxKind::ObjectLiteralExpression
        | SyntaxKind::PropertyAccessExpression
        | SyntaxKind::ElementAccessExpression
        | SyntaxKind::CallExpression
        | SyntaxKind::NewExpression
        | SyntaxKind::TaggedTemplateExpression
        | SyntaxKind::AsExpression
        | SyntaxKind::TypeAssertionExpression
        | SyntaxKind::SatisfiesExpression
        | SyntaxKind::NonNullExpression
        | SyntaxKind::ParenthesizedExpression
        | SyntaxKind::FunctionExpression
        | SyntaxKind::ClassExpression
        | SyntaxKind::ArrowFunction
        | SyntaxKind::VoidExpression
        | SyntaxKind::DeleteExpression
        | SyntaxKind::TypeOfExpression
        | SyntaxKind::PrefixUnaryExpression
        | SyntaxKind::PostfixUnaryExpression
        | SyntaxKind::BinaryExpression
        | SyntaxKind::ConditionalExpression
        | SyntaxKind::SpreadElement
        | SyntaxKind::TemplateExpression
        | SyntaxKind::OmittedExpression
        | SyntaxKind::JsxElement
        | SyntaxKind::JsxSelfClosingElement
        | SyntaxKind::JsxFragment
        | SyntaxKind::YieldExpression
        | SyntaxKind::AwaitExpression => true,
        SyntaxKind::MetaProperty => node.parent().map_or(true, |parent| {
            !(is_import_call(&parent)
                && parent
                    .expression()
                    .is_some_and(|e| std::ptr::eq(e.as_ref(), node)))
        }),
        SyntaxKind::ExpressionWithTypeArguments => {
            node.parent()
                .as_ref()
                .is_some_and(|p| !is_heritage_clause(p))
        }
        SyntaxKind::QualifiedName => {
            let mut current = node.parent();
            while current.as_ref().is_some_and(|p| p.kind == SyntaxKind::QualifiedName) {
                current = current.and_then(|p| p.parent());
            }
            match current {
                Some(p) => {
                    is_type_query_node(&p)
                        || crate::ast::utilities_types::is_jsdoc_link_like(&p)
                        || is_jsdoc_name_reference(&p)
                        || crate::ast::utilities_misc::is_jsx_tag_name(&p)
                }
                None => false,
            }
        }
        SyntaxKind::PrivateIdentifier => {
            node.parent().as_ref().is_some_and(|p| {
                matches!(
                    &p.data,
                    NodeData::BinaryExpression(be)
                        if std::ptr::eq(be.left.as_ref(), node)
                            && be.operator_token.kind == SyntaxKind::InKeyword
                )
            })
        }
        SyntaxKind::Identifier => {
            if node.parent().as_ref().is_some_and(|p| {
                is_type_query_node(p)
                    || crate::ast::utilities_types::is_jsdoc_link_like(p)
                    || is_jsdoc_name_reference(p)
            }) {
                return true;
            }
            is_in_expression_context(node)
        }
        SyntaxKind::NumericLiteral
        | SyntaxKind::BigIntLiteral
        | SyntaxKind::StringLiteral
        | SyntaxKind::NoSubstitutionTemplateLiteral
        | SyntaxKind::ThisKeyword => is_in_expression_context(node),
        _ => false,
    }
}

fn node_body(node: &Node) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("node_body"); 
    match &node.data {
        NodeData::FunctionDeclaration(d) => d.body.clone(),
        NodeData::FunctionExpression(d) => Some(d.body.clone()),
        NodeData::MethodDeclaration(d) => d.body.clone(),
        NodeData::ConstructorDeclaration(d) => d.body.clone(),
        NodeData::GetAccessorDeclaration(d) => d.body.clone(),
        NodeData::SetAccessorDeclaration(d) => d.body.clone(),
        NodeData::ArrowFunction(d) => Some(d.body.clone()),
        _ => None,
    }
}

fn node_members(node: &Node) -> &[Arc<Node>] { ::tsox_core::fntrace::enter("node_members"); 
    match &node.data {
        NodeData::ClassDeclaration(d) => &d.members.nodes,
        NodeData::ClassExpression(d) => &d.members.nodes,
        NodeData::InterfaceDeclaration(d) => &d.members.nodes,
        NodeData::EnumDeclaration(d) => &d.members.nodes,
        NodeData::TypeLiteralNode(d) => &d.members.nodes,
        NodeData::MappedTypeNode(d) => d.members.as_deref().map(|m| m.nodes.as_slice()).unwrap_or(&[]),
        _ => &[],
    }
}

fn get_this_parameter(signature: &Node) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_this_parameter"); 
    let parameters = node_parameters(signature)?;
    let first = parameters.nodes.first()?;
    is_this_parameter(first).then(|| Arc::clone(first))
}

impl NodeFactory {
    pub fn update_type_parameter_declaration(
        &self,
        node: &crate::ast::node_data_generated::TypeParameterDeclarationData,
        modifiers: Option<Arc<ModifierList>>,
        name: Option<Arc<Node>>,
        constraint: Option<Arc<Node>>,
        expression: Option<Arc<Node>>,
        default_type: Option<Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_type_parameter_declaration"); 
        let _ = self;
        Arc::new(Node::new(
            SyntaxKind::TypeParameter,
            NodeData::TypeParameterDeclaration(
                crate::ast::node_data_generated::TypeParameterDeclarationData {
                    modifiers,
                    name: name.unwrap_or_else(|| node.name.clone()),
                    constraint: constraint.or_else(|| node.constraint.clone()),
                    expression: expression.or_else(|| node.expression.clone()),
                    default_type: default_type.or_else(|| node.default_type.clone()),
                },
            ),
        ))
    }

    pub fn update_parameter_declaration(
        &self,
        node: &crate::ast::node_data_generated::ParameterDeclarationData,
        modifiers: Option<Arc<ModifierList>>,
        dot_dot_dot_token: Option<Arc<Node>>,
        name: Option<Arc<Node>>,
        question_token: Option<Arc<Node>>,
        type_node: Option<Arc<Node>>,
        initializer: Option<Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_parameter_declaration"); 
        let _ = self;
        Arc::new(Node::new(
            SyntaxKind::Parameter,
            NodeData::ParameterDeclaration(
                crate::ast::node_data_generated::ParameterDeclarationData {
                    modifiers,
                    dot_dot_dot_token: dot_dot_dot_token.or_else(|| node.dot_dot_dot_token.clone()),
                    name: name.unwrap_or_else(|| node.name.clone()),
                    question_token: question_token.or_else(|| node.question_token.clone()),
                    type_node: type_node.or_else(|| node.type_node.clone()),
                    initializer: initializer.or_else(|| node.initializer.clone()),
                },
            ),
        ))
    }

    pub fn update_property_signature_declaration(
        &self,
        node: &crate::ast::node_data_generated::PropertySignatureDeclarationData,
        modifiers: Option<Arc<ModifierList>>,
        name: Option<Arc<Node>>,
        postfix_token: Option<Arc<Node>>,
        type_node: Option<Arc<Node>>,
        initializer: Option<Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_property_signature_declaration"); 
        let _ = self;
        Arc::new(Node::new(
            SyntaxKind::PropertySignature,
            NodeData::PropertySignatureDeclaration(
                crate::ast::node_data_generated::PropertySignatureDeclarationData {
                    modifiers,
                    name: name.unwrap_or_else(|| node.name.clone()),
                    postfix_token: postfix_token.or_else(|| node.postfix_token.clone()),
                    type_node: type_node.unwrap_or_else(|| node.type_node.clone()),
                    initializer: initializer.unwrap_or_else(|| node.initializer.clone()),
                },
            ),
        ))
    }

    pub fn update_declaration_modifiers(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_declaration_modifiers"); 
        let cloned = crate::ast::deep_clone_node(node);
        let Ok(mut owned) = Arc::try_unwrap(cloned) else {
            return Arc::clone(node);
        };
        match &mut owned.data {
            NodeData::PropertyDeclaration(d) => d.modifiers = modifiers,
            NodeData::MethodSignatureDeclaration(d) => d.modifiers = modifiers,
            NodeData::MethodDeclaration(d) => d.modifiers = modifiers,
            NodeData::ConstructorDeclaration(d) => d.modifiers = modifiers,
            NodeData::GetAccessorDeclaration(d) => d.modifiers = modifiers,
            NodeData::SetAccessorDeclaration(d) => d.modifiers = modifiers,
            NodeData::IndexSignatureDeclaration(d) => d.modifiers = modifiers,
            NodeData::FunctionExpression(d) => d.modifiers = modifiers,
            NodeData::ArrowFunction(d) => d.modifiers = modifiers,
            NodeData::ClassExpression(d) => d.modifiers = modifiers,
            NodeData::VariableStatement(d) => d.modifiers = modifiers,
            NodeData::FunctionDeclaration(d) => d.modifiers = modifiers,
            NodeData::ClassDeclaration(d) => d.modifiers = modifiers,
            NodeData::EnumDeclaration(d) => d.modifiers = modifiers,
            NodeData::ModuleDeclaration(d) => d.modifiers = modifiers,
            NodeData::ImportEqualsDeclaration(d) => d.modifiers = modifiers,
            NodeData::ImportDeclaration(d) => d.modifiers = modifiers,
            NodeData::ExportDeclaration(d) => d.modifiers = modifiers,
            NodeData::InterfaceDeclaration(d) => d.modifiers = modifiers,
            NodeData::TypeAliasDeclaration(d) => d.modifiers = modifiers,
            _ => {}
        }
        Arc::new(owned)
    }
}

pub fn replace_modifiers(
    factory: &NodeFactory,
    node: &Arc<Node>,
    modifier_array: Option<Arc<ModifierList>>,
) -> Arc<Node> { ::tsox_core::fntrace::enter("replace_modifiers"); 
    match node.kind {
        SyntaxKind::TypeParameter => match &node.data {
            NodeData::TypeParameterDeclaration(d) => factory.update_type_parameter_declaration(
                d,
                modifier_array,
                Some(d.name.clone()),
                d.constraint.clone(),
                d.expression.clone(),
                d.default_type.clone(),
            ),
            _ => node.clone(),
        },
        SyntaxKind::Parameter => match &node.data {
            NodeData::ParameterDeclaration(d) => factory.update_parameter_declaration(
                d,
                modifier_array,
                d.dot_dot_dot_token.clone(),
                Some(d.name.clone()),
                d.question_token.clone(),
                d.type_node.clone(),
                d.initializer.clone(),
            ),
            _ => node.clone(),
        },
        SyntaxKind::PropertySignature => match &node.data {
            NodeData::PropertySignatureDeclaration(d) => factory.update_property_signature_declaration(
                d,
                modifier_array,
                Some(d.name.clone()),
                d.postfix_token.clone(),
                Some(d.type_node.clone()),
                Some(d.initializer.clone()),
            ),
            _ => node.clone(),
        },
        SyntaxKind::PropertyDeclaration
        | SyntaxKind::MethodSignature
        | SyntaxKind::MethodDeclaration
        | SyntaxKind::Constructor
        | SyntaxKind::GetAccessor
        | SyntaxKind::SetAccessor
        | SyntaxKind::IndexSignature
        | SyntaxKind::FunctionExpression
        | SyntaxKind::ArrowFunction
        | SyntaxKind::ClassExpression
        | SyntaxKind::VariableStatement
        | SyntaxKind::FunctionDeclaration
        | SyntaxKind::ClassDeclaration
        | SyntaxKind::EnumDeclaration
        | SyntaxKind::ModuleDeclaration
        | SyntaxKind::ImportEqualsDeclaration
        | SyntaxKind::ImportDeclaration
        | SyntaxKind::ExportDeclaration
        | SyntaxKind::InterfaceDeclaration
        | SyntaxKind::TypeAliasDeclaration => factory.update_declaration_modifiers(node, modifier_array),
        _ => node.clone(),
    }
}
