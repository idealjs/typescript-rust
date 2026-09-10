#![allow(dead_code)]

use std::sync::Arc;

use tsox_checker::checker::Checker;
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::Node;
use tsox_frontend::ast::SourceFile;
use tsox_frontend::ast::Symbol;

use super::language_service::LanguageService;

pub struct CompletionsFromTypes {
    pub is_new_identifier: bool,
}

pub struct CompletionsFromProperties {
    pub symbols: Vec<Arc<Symbol>>,
    pub has_index_signature: bool,
}

pub struct PathCompletion {
    pub name: String,
    pub extension: String,
    pub text_range: TextRange,
}

pub struct StringLiteralCompletions {
    pub from_types: Option<CompletionsFromTypes>,
    pub from_properties: Option<CompletionsFromProperties>,
    pub from_paths: Vec<PathCompletion>,
}

/// 字符串字面量位置的联合成员/属性名补全标签（label 级）。
/// Go getStringLiteralCompletionEntries → fromUnionableLiteralType 的两个主形态：
/// 调用实参（形参约束的字面量并集）与索引访问（对象属性名）
pub fn string_literal_completion_labels(
    checker: &mut Checker,
    node: &Arc<Node>,
    position: usize,
) -> Option<Vec<String>> {
    use tsox_frontend::ast::NodeData;

    // 定位包含位置的最内字符串字面量
    let mut lit: Option<Arc<Node>> = None;
    let mut cur = Some(Arc::clone(node));
    while let Some(c) = cur {
        if c.kind == tsox_frontend::ast::SyntaxKind::StringLiteral {
            lit = Some(Arc::clone(&c));
            break;
        }
        if c.pos() > position || c.end() <= position {
            break;
        }
        let mut next: Option<Arc<Node>> = None;
        tsox_frontend::ast::node_data_generated::for_each_child(&c, |ch| {
            if ch.pos() <= position && position <= ch.end() {
                next = Some(Arc::clone(ch));
                true
            } else {
                false
            }
        });
        cur = next;
    }
    let lit = lit?;
    let parent = lit.parent.clone()?;

    match &parent.data {
        // f("...")：按实参位取形参约束
        NodeData::CallExpression(d) => {
            let idx = d
                .arguments
                .nodes
                .iter()
                .position(|a| Arc::ptr_eq(a, &lit))?;
            let callee_type = checker.get_type_of_node(&d.expression);
            let structured = callee_type.as_structured()?;
            let sigs = structured.call_signatures();
            let sig = sigs.first()?.clone();
            let param = sig.parameters.get(idx).cloned()?;
            let t = checker.get_type_of_symbol(&param);
            Some(literal_union_labels(checker, &t))
        }
        // obj["..."]：对象属性名
        NodeData::IndexedAccessTypeNode(d) if Arc::ptr_eq(&d.index_type, &lit) => {
            let obj_t = checker.get_type_from_type_node(&d.object_type);
            let props = checker.get_apparent_properties(&obj_t);
            Some(props.into_iter().map(|p| p.name.clone()).collect())
        }
        _ => None,
    }
}

fn literal_union_labels(
    checker: &mut Checker,
    t: &Arc<tsox_checker::checker::types::Type>,
) -> Vec<String> {
    use tsox_checker::checker::types::TypeFlags;
    let mut out = Vec::new();
    let mut stack = vec![Arc::clone(t)];
    while let Some(ty) = stack.pop() {
        if ty.flags.intersects(TypeFlags::StringLiteral) {
            if let Some(v) = ty.literal_value() {
                if let tsox_checker::checker::types::LiteralValue::String(s) = v {
                    out.push(s.clone());
                }
            }
        } else if ty.is_union() {
            stack.extend(ty.types().unwrap_or_default().to_vec());
        } else if ty.flags.intersects(TypeFlags::TypeParameter) {
            // 泛型形参（K extends keyof Foo）：Go GetTypeArgumentConstraint
            if let Some(c) = checker.get_constraint_of_type_parameter(&ty) {
                stack.push(c);
            }
        }
    }
    out
}
