#![allow(unused_imports)]

use super::*;

pub(crate) fn object_literal_is_destructuring_target(literal: &Arc<Node>) -> bool {
    let Some(parent) = literal.parent.as_ref() else {
        return false;
    };
    match parent.kind {
        SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement => true,
        SyntaxKind::BinaryExpression => {
            matches!(&parent.data, crate::ast::NodeData::BinaryExpression(bin)
            if bin.operator_token.kind == SyntaxKind::EqualsToken
                && std::ptr::eq(
                    bin.left.as_ref() as *const Node,
                    literal.as_ref() as *const Node
                ))
        }
        SyntaxKind::ParenthesizedExpression => object_literal_is_destructuring_target(parent),
        _ => false,
    }
}

pub(crate) fn is_assignment_target(node: &Arc<Node>) -> bool {
    let Some(parent) = node.parent.as_ref() else {
        return false;
    };

    if parent.kind == SyntaxKind::BindingElement {
        if let crate::ast::NodeData::BindingElement(be) = &parent.data {
            if let Some(name) = &be.name {
                return std::ptr::eq(name.as_ref() as *const Node, node.as_ref() as *const Node)
                    && !matches!(
                        name.kind,
                        SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern
                    );
            }
        }
        return false;
    }

    if parent.kind == SyntaxKind::ShorthandPropertyAssignment {
        if let crate::ast::NodeData::ShorthandPropertyAssignment(sa) = &parent.data {
            let name_is_node = std::ptr::eq(
                sa.name.as_ref() as *const Node,
                node.as_ref() as *const Node,
            );
            let literal = parent.parent.as_ref();
            if name_is_node
                && literal.is_some_and(|lit| {
                    lit.kind == SyntaxKind::ObjectLiteralExpression
                        && object_literal_is_destructuring_target(lit)
                })
            {
                return true;
            }
        }
        return false;
    }
    if parent.kind != SyntaxKind::BinaryExpression {
        return false;
    }
    let crate::ast::NodeData::BinaryExpression(bin) = &parent.data else {
        return false;
    };
    if !is_compound_or_simple_assignment(bin.operator_token.kind) {
        return false;
    }

    std::ptr::eq(
        bin.left.as_ref() as *const Node,
        node.as_ref() as *const Node,
    )
}

pub(crate) fn is_let_or_const_declaration(declaration: &Arc<Node>) -> bool {
    if let Some(parent) = declaration.parent.as_ref() {
        if parent.kind == SyntaxKind::VariableDeclarationList {
            return parent.flags.intersects(NodeFlags::Let | NodeFlags::Const);
        }
    }

    true
}

pub(crate) fn type_contains_undefined(t: &Arc<Type>) -> bool {
    if t.flags.contains(TypeFlags::Undefined) {
        return true;
    }
    if t.flags.contains(TypeFlags::Union) {
        if let TypeData::Union(u) = &t.data {
            return u
                .union_or_intersection
                .types
                .iter()
                .any(type_contains_undefined);
        }
    }
    false
}

pub(crate) fn type_is_possibly_undefined(t: &Arc<Type>) -> bool {
    if t.flags.intersects(TypeFlags::Undefined | TypeFlags::Null) {
        return true;
    }
    if t.flags.contains(TypeFlags::Union) {
        if let TypeData::Union(u) = &t.data {
            return u
                .union_or_intersection
                .types
                .iter()
                .any(|ct| ct.flags.intersects(TypeFlags::Undefined | TypeFlags::Null));
        }
    }
    false
}

pub(crate) fn type_includes_undefined_only(t: &Arc<Type>) -> bool {
    if t.flags.contains(TypeFlags::Undefined) {
        return true;
    }
    if t.flags.contains(TypeFlags::Union) {
        if let TypeData::Union(u) = &t.data {
            return u
                .union_or_intersection
                .types
                .iter()
                .any(|ct| ct.flags.contains(TypeFlags::Undefined));
        }
    }
    false
}

pub(crate) fn type_includes_null_only(t: &Arc<Type>) -> bool {
    if t.flags.contains(TypeFlags::Null) {
        return true;
    }
    if t.flags.contains(TypeFlags::Union) {
        if let TypeData::Union(u) = &t.data {
            return u
                .union_or_intersection
                .types
                .iter()
                .any(|ct| ct.flags.contains(TypeFlags::Null));
        }
    }
    false
}

pub(crate) fn is_entity_name_expression(node: &Arc<Node>) -> bool {
    match node.kind {
        SyntaxKind::Identifier => true,
        SyntaxKind::PropertyAccessExpression => match &node.data {
            crate::ast::NodeData::PropertyAccessExpression(data) => {
                is_entity_name_expression(&data.expression)
            }
            _ => false,
        },
        _ => false,
    }
}

pub(crate) fn is_compound_or_simple_assignment(kind: SyntaxKind) -> bool {
    use SyntaxKind::*;
    matches!(
        kind,
        EqualsToken
            | PlusEqualsToken
            | MinusEqualsToken
            | AsteriskEqualsToken
            | AsteriskAsteriskEqualsToken
            | SlashEqualsToken
            | PercentEqualsToken
            | LessThanLessThanEqualsToken
            | GreaterThanGreaterThanEqualsToken
            | GreaterThanGreaterThanGreaterThanEqualsToken
            | AmpersandEqualsToken
            | BarEqualsToken
            | CaretEqualsToken
            | BarBarEqualsToken
            | AmpersandAmpersandEqualsToken
            | QuestionQuestionEqualsToken
    )
}

pub(crate) fn flatten_union_leaves<'a>(t: &'a Arc<Type>, leaves: &mut Vec<&'a Arc<Type>>) {
    match t.as_union_or_intersection() {
        Some(u) => {
            for m in &u.types {
                flatten_union_leaves(m, leaves);
            }
        }
        None => leaves.push(t),
    }
}

pub(crate) fn class_declaration_name(class: &Arc<Node>) -> Option<String> {
    if let crate::ast::NodeData::ClassDeclaration(d) = &class.data {
        return d.name.as_ref().map(|n| n.text().to_string());
    }
    None
}

pub(crate) fn prop_decl_has_initializer(decl: &Arc<Node>) -> bool {
    matches!(&decl.data, crate::ast::NodeData::PropertyDeclaration(d) if d.initializer.is_some())
}

pub(crate) fn later_sibling_property(node: &Arc<Node>, prop_decl: &Arc<Node>) -> bool {
    let mut cur = node.parent.as_ref();
    while let Some(a) = cur {
        if a.kind == SyntaxKind::PropertyDeclaration {
            return prop_decl.loc.pos() > a.loc.pos();
        }
        cur = a.parent.as_ref();
    }
    false
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ModuleMemberLookup {
    Found,

    LocalNotExported,
    Missing,
}

#[allow(dead_code)]
pub(crate) fn body_assigns_this_property(n: &Arc<Node>, name: &str) -> bool {
    match &n.data {
        crate::ast::NodeData::BinaryExpression(b)
            if b.operator_token.kind == SyntaxKind::EqualsToken =>
        {
            if let crate::ast::NodeData::PropertyAccessExpression(pa) = &b.left.data
                && pa.expression.kind == SyntaxKind::ThisKeyword
                && pa.name.kind == SyntaxKind::Identifier
                && pa.name.text() == name
            {
                return true;
            }
        }

        crate::ast::NodeData::FunctionDeclaration(_)
        | crate::ast::NodeData::FunctionExpression(_)
        | crate::ast::NodeData::ArrowFunction(_) => return false,
        _ => {}
    }
    let mut found = false;
    crate::ast::node_data_generated::for_each_child(n, |child| {
        if body_assigns_this_property(child, name) {
            found = true;
            true
        } else {
            false
        }
    });
    found
}

impl std::fmt::Debug for Checker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Checker")
            .field("id", &self.id)
            .field("type_count", &self.type_count)
            .field("symbol_count", &self.symbol_count)
            .field("files", &self.files.len())
            .finish()
    }
}

impl Checker {
    pub(crate) fn attach_explicit_type_arguments_cached(
        &mut self,
        t: &Arc<Type>,
        args: Vec<Arc<Type>>,
    ) -> Arc<Type> {
        let mut key = Vec::with_capacity(args.len() + 1);
        key.push(t.id as usize);
        key.extend(args.iter().map(|a| a.id as usize));
        if let Some(cached) = self.attached_type_args_cache.get(&key) {
            return Arc::clone(&cached.2);
        }
        let rebuilt = attach_explicit_type_arguments(t, args.clone());
        self.attached_type_args_cache
            .insert(key, (Arc::clone(t), args, Arc::clone(&rebuilt)));
        rebuilt
    }
}
