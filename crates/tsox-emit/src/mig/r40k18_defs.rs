#![allow(unused_imports, dead_code)]

use std::sync::Arc;

use tsox_checker::checker::mig::m2d::EmitResolver;
use tsox_core::jsnum::{Number, PseudoBigInt};
use tsox_frontend::ast::node_data_generated::{self as ndg, NodeData};
use tsox_frontend::ast::{get_source_file_of_node, Node, NodeFlags, SyntaxKind};
use tsox_frontend::evaluator::{evaluate_expression, EvalResult, EvalValue};
use tsox_frontend::format::mig::m4o::EmitFlags;

use crate::mig::m4i_13::ConstantValue;
use crate::printer::EmitContext;

const MAX_ENUM_MEMBER_DEPTH: u32 = 32;

pub trait R40K18EmitResolverExt {
    fn get_constant_value(&self, node: &Arc<Node>) -> Option<ConstantValue>;
}

impl R40K18EmitResolverExt for EmitResolver {
    fn get_constant_value(&self, node: &Arc<Node>) -> Option<ConstantValue> {
        if node.kind == SyntaxKind::EnumMember {
            return evaluate_enum_member(node, None);
        }
        let (enum_name, member_name) = enum_member_access_path(node)?;
        let source_file = get_source_file_of_node(node)?;
        let enums = collect_enum_declarations(&source_file);
        match enum_name {
            Some(enum_name) => {
                let enum_decl = enums
                    .iter()
                    .find(|e| enum_declaration_name(e).as_deref() == Some(enum_name.as_str()))?;
                if !is_enum_const(enum_decl) {
                    return None;
                }
                enum_member_value_by_name(enum_decl, &member_name, &enums)
            }
            None => enums
                .iter()
                .filter(|e| is_enum_const(e))
                .find_map(|e| enum_member_value_by_name(e, &member_name, &enums)),
        }
    }
}

fn enum_member_access_path(node: &Arc<Node>) -> Option<(Option<String>, String)> {
    match &node.data {
        NodeData::PropertyAccessExpression(d) => {
            Some((entity_name_text(&d.expression), d.name.text().to_string()))
        }
        NodeData::ElementAccessExpression(d) => {
            let argument = &d.argument_expression;
            if argument.kind != SyntaxKind::StringLiteral {
                return None;
            }
            let NodeData::StringLiteral(literal) = &argument.data else {
                return None;
            };
            Some((entity_name_text(&d.expression), literal.text.clone()))
        }
        _ => None,
    }
}

fn entity_name_text(node: &Arc<Node>) -> Option<String> {
    match &node.data {
        NodeData::Identifier(d) => Some(d.text.clone()),
        NodeData::PropertyAccessExpression(d) => Some(d.name.text().to_string()),
        _ => None,
    }
}

fn collect_enum_declarations(node: &Arc<Node>) -> Vec<Arc<Node>> {
    let mut result = Vec::new();
    collect_enum_declarations_into(node, &mut result);
    result
}

fn collect_enum_declarations_into(node: &Arc<Node>, result: &mut Vec<Arc<Node>>) {
    if node.kind == SyntaxKind::EnumDeclaration {
        result.push(Arc::clone(node));
    }
    ndg::for_each_child(node, |child| {
        collect_enum_declarations_into(child, result);
        true
    });
}

fn enum_declaration_name(enum_decl: &Arc<Node>) -> Option<String> {
    match &enum_decl.data {
        NodeData::EnumDeclaration(d) => Some(d.name.text().to_string()),
        _ => None,
    }
}

fn is_enum_const(enum_decl: &Arc<Node>) -> bool {
    enum_decl.flags.contains(NodeFlags::Const)
}

fn enum_member_value_by_name(
    enum_decl: &Arc<Node>,
    member_name: &str,
    enums: &[Arc<Node>],
) -> Option<ConstantValue> {
    let NodeData::EnumDeclaration(d) = &enum_decl.data else {
        return None;
    };
    let mut auto_value: Option<f64> = None;
    for member in &d.members.nodes {
        let value = evaluate_enum_member_with(member, enums, auto_value, 0);
        if member_name_text(member).as_deref() == Some(member_name) {
            return value;
        }
        if let Some(ConstantValue::Number(n)) = &value {
            auto_value = Some(n.0);
        }
    }
    None
}

fn member_name_text(member: &Arc<Node>) -> Option<String> {
    match &member.data {
        NodeData::EnumMember(d) => Some(d.name.text().to_string()),
        _ => None,
    }
}

fn evaluate_enum_member(member: &Arc<Node>, _parent: Option<&Arc<Node>>) -> Option<ConstantValue> {
    let source_file = get_source_file_of_node(member)?;
    let enums = collect_enum_declarations(&source_file);
    evaluate_enum_member_with(member, &enums, None, 0)
}

fn evaluate_enum_member_with(
    member: &Arc<Node>,
    enums: &[Arc<Node>],
    auto_value: Option<f64>,
    depth: u32,
) -> Option<ConstantValue> {
    let NodeData::EnumMember(d) = &member.data else {
        return None;
    };
    match &d.initializer {
        Some(initializer) => {
            let mut resolver = EnumMemberResolver { enums, depth };
            let result = evaluate_expression(initializer, None, &mut |expr, location| {
                resolver.resolve(expr, location)
            });
            to_constant_value(result.value)
        }
        None => Some(ConstantValue::Number(Number(match auto_value {
            Some(previous) => previous + 1.0,
            None => 0.0,
        }))),
    }
}

fn to_constant_value(value: Option<EvalValue>) -> Option<ConstantValue> {
    match value {
        Some(EvalValue::Number(v)) => Some(ConstantValue::Number(v)),
        Some(EvalValue::String(v)) => Some(ConstantValue::String(v)),
        Some(EvalValue::BigInt(v)) => Some(ConstantValue::PseudoBigInt(v)),
        _ => None,
    }
}

struct EnumMemberResolver<'a> {
    enums: &'a [Arc<Node>],
    depth: u32,
}

impl EnumMemberResolver<'_> {
    fn resolve(&mut self, expr: &Arc<Node>, _location: Option<&Arc<Node>>) -> EvalResult {
        if self.depth >= MAX_ENUM_MEMBER_DEPTH {
            return EvalResult::none();
        }
        let Some(member_name) = entity_name_text(expr) else {
            return EvalResult::none();
        };
        for enum_decl in self.enums {
            if !is_enum_const(enum_decl) {
                continue;
            }
            if let Some(value) =
                enum_member_value_by_name(enum_decl, &member_name, self.enums)
            {
                return EvalResult::new(to_eval_value(&value), false, false, false);
            }
        }
        EvalResult::none()
    }
}

fn to_eval_value(value: &ConstantValue) -> Option<EvalValue> {
    match value {
        ConstantValue::Number(v) => Some(EvalValue::Number(*v)),
        ConstantValue::String(v) => Some(EvalValue::String(v.clone())),
        ConstantValue::PseudoBigInt(v) => Some(EvalValue::BigInt(v.clone())),
    }
}

pub trait R40K18EmitContextExt {
    fn set_emit_flags_shared(&self, node: &Arc<Node>, flags: EmitFlags);
}

impl R40K18EmitContextExt for EmitContext {
    fn set_emit_flags_shared(&self, node: &Arc<Node>, flags: EmitFlags) {
        self.emit_nodes_get_mut(node).emit_flags = flags;
    }
}
