use std::sync::Arc;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_frontend::ast::node_data_generated::{CallExpressionData, ClassDeclarationData, ElementAccessExpressionData, PropertyDeclarationData};
use tsox_frontend::ast::{Node, NodeFlags, SyntaxKind};
use crate::printer::EmitContext;
use crate::mig::m4n_8::r36k34_defs::NodeR36k34Accessors;
use crate::mig::m4j_2::extract_modifiers;
use tsox_frontend::ast::node_flags::ModifierFlags;
use tsox_core::collections::ordered_set::OrderedSet;
use tsox_frontend::ast::visitor::NodeVisitor;
pub fn convert_class_declaration_to_class_expression(
    emit_context: &EmitContext,
    node: &Arc<Node>,
) -> Arc<Node> {
    let data = node.as_class_declaration();
        let updated = emit_context.factory().new_class_expression(
            extract_modifiers(
                &Arc::new(emit_context.clone()),
                node.modifiers().map(|v| &**v),
                ModifierFlags::Export | ModifierFlags::Default,
            )
            .map(Arc::new),
        node.name().cloned(),
        data.type_parameters.clone(),
        data.heritage_clauses.clone(),
        data.members.clone(),
    );
    emit_context.set_original(&updated, node);
    let mut updated = updated;
    if let Some(u) = Arc::get_mut(&mut updated) {
        u.loc = node.loc;
    }
    updated
}

pub fn create_not_null_condition(
    emit_context: &EmitContext,
    left: Arc<Node>,
    right: Arc<Node>,
    invert: bool,
) -> Arc<Node> {
    let (token, op) = if invert {
        (SyntaxKind::EqualsEqualsEqualsToken, SyntaxKind::BarBarToken)
    } else {
        (
            SyntaxKind::ExclamationEqualsEqualsToken,
            SyntaxKind::AmpersandAmpersandToken,
        )
    };
    let f = emit_context.factory();
    let left_check = f.new_binary_expression(
        None,
        &left,
        None,
        &f.new_token(token),
        &f.new_keyword_expression(SyntaxKind::NullKeyword),
    );
    let right_check = f.new_binary_expression(
        None,
        &right,
        None,
        &f.new_token(token),
        &f.new_void_zero_expression(),
    );
    f.new_binary_expression(None, &left_check, None, &f.new_token(op), &right_check)
}

pub struct SuperAccessState {
    emit_context: EmitContext,
    captured_super_properties: Option<OrderedSet<String>>,
    has_super_element_access: bool,
    has_super_property_assignment: bool,
    super_binding: Arc<Node>,
    super_index_binding: Arc<Node>,
    super_access_visitor: NodeVisitor,
}

