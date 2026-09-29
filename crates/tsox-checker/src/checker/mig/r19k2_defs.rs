#![allow(unused_imports)]

use crate::checker::checker_checker::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering as AtomicOrdering};
use tsox_frontend::ast::{Diagnostic, Node, Symbol};

use crate::checker::relater_relation::{Relation, RelationKind};
use crate::checker::types_type_id::TypeFlags;

pub const TYPE_FLAGS_INSTANTIABLE_NON_PRIMITIVE: TypeFlags = crate::checker::types_type_id::TYPE_FLAGS_TYPE_VARIABLE
    .union(TypeFlags::Conditional)
    .union(TypeFlags::Substitution);
pub const TYPE_FLAGS_INSTANTIABLE_PRIMITIVE: TypeFlags = TypeFlags::Index
    .union(TypeFlags::TemplateLiteral)
    .union(TypeFlags::StringMapping);
pub const TYPE_FLAGS_INSTANTIABLE: TypeFlags =
    TYPE_FLAGS_INSTANTIABLE_NON_PRIMITIVE.union(TYPE_FLAGS_INSTANTIABLE_PRIMITIVE);
pub const TYPE_FLAGS_INCLUDES_MASK: TypeFlags = TypeFlags::Any
    .union(TypeFlags::Unknown)
    .union(crate::checker::types_type_id::TYPE_FLAGS_PRIMITIVE)
    .union(TypeFlags::Never)
    .union(TypeFlags::Object)
    .union(TypeFlags::Union)
    .union(TypeFlags::Intersection)
    .union(TypeFlags::NonPrimitive)
    .union(TypeFlags::TemplateLiteral)
    .union(TypeFlags::StringMapping);
pub const TYPE_FLAGS_INCLUDES_MISSING_TYPE: TypeFlags = TypeFlags::TypeParameter;
pub const TYPE_FLAGS_INCLUDES_NON_WIDENING_TYPE: TypeFlags = TypeFlags::Index;
pub const TYPE_FLAGS_INCLUDES_WILDCARD: TypeFlags = TypeFlags::IndexedAccess;
pub const TYPE_FLAGS_INCLUDES_EMPTY_OBJECT: TypeFlags = TypeFlags::Conditional;
pub const TYPE_FLAGS_INCLUDES_INSTANTIABLE: TypeFlags = TypeFlags::Substitution;
pub const TYPE_FLAGS_INCLUDES_CONSTRAINED_TYPE_VARIABLE: TypeFlags = TypeFlags::Reserved1;
pub const TYPE_FLAGS_INCLUDES_ERROR: TypeFlags = TypeFlags::Reserved2;

static NEXT_CHECKER_ID: AtomicU32 = AtomicU32::new(1);

pub fn next_checker_id() -> u32 {
    NEXT_CHECKER_ID.fetch_add(1, AtomicOrdering::Relaxed)
}

pub fn get_adjusted_node_for_error(node: &Arc<Node>) -> Option<Arc<Node>> {
    Some(Arc::clone(node))
}

pub fn is_static_private_identifier_property(s: &Arc<Symbol>) -> bool {
    let Some(declaration) = &s.value_declaration else {
        return false;
    };
    tsox_frontend::ast::mig::m3g_2::is_private_identifier_class_element_declaration(declaration)
        && crate::checker::checker_ast_get_combined_modifier_flags::ast_get_combined_modifier_flags(
            declaration,
        )
        .contains(ModifierFlags::Static)
}

pub fn is_export_assignment_expression_name(node: &Arc<Node>) -> bool {
    let mut current = Arc::clone(node);
    while let Some(parent) = current.parent() {
        if parent.kind != SyntaxKind::PropertyAccessExpression
            && parent.kind != SyntaxKind::QualifiedName
        {
            break;
        }
        current = parent;
    }
    let Some(parent) = current.parent() else {
        return false;
    };
    parent.kind == SyntaxKind::ExportAssignment
        && parent
            .expression()
            .is_some_and(|e| Arc::ptr_eq(&e, &current))
}

pub fn is_literal_import_type_node(node: &Arc<Node>) -> bool {
    node.kind == SyntaxKind::ImportType
        && matches!(
            &node.data,
            tsox_frontend::ast::NodeData::ImportTypeNode(d) if d.is_type_of
        )
}

pub fn compare_diagnostics(a: &Diagnostic, b: &Diagnostic) -> i32 {
    match tsox_frontend::ast::mig::m3d_2::compare_diagnostics(a, b) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

impl Checker {
    pub fn assignable_relation(&self) -> Relation {
        Relation::new(RelationKind::Assignable)
    }

    pub(crate) fn symbol_of_node(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        self.symbol_node_links
            .get(node)
            .and_then(|l| l.resolved_symbol.clone())
    }
}

pub fn symbol_of_node(node: &Arc<Node>) -> Option<Arc<Symbol>> {
    let ptr = crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr();
    if ptr.is_null() {
        return None;
    }
    let c = unsafe { &*ptr };
    Checker::symbol_of_node(c, node)
}

use tsox_frontend::ast::SyntaxKind;
