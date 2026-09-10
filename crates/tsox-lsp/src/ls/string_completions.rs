#![allow(dead_code)]

use std::sync::Arc;

use tsox_checker::checker::Checker;
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::Node;
use tsox_frontend::ast::SyntaxKind;
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
        // f("...")：按实参位取形参约束；泛型签名先由其余实参推断类型参数
        // （Go getStringLiteralCompletionsFromSignature）
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
            if !sig.type_parameters.is_empty() {
                let mut inferred =
                    checker.infer_call_type_arguments(&parent, &sig, &d.arguments.nodes);
                // 推断值可含未回填的其它类型参数（K=keyof T）：对推断结果自身
                // 再过一遍替换链闭环
                for _ in 0..2 {
                    let current = inferred.clone();
                    inferred = inferred
                        .into_iter()
                        .map(|x| {
                            checker.substitute_infer_type_parameters(
                                &x,
                                &sig.type_parameters,
                                &current,
                            )
                        })
                        .collect::<Vec<_>>();
                }
                let inst = checker.substitute_infer_type_parameters(
                    &t,
                    &sig.type_parameters,
                    &inferred,
                );
                return Some(literal_union_labels(checker, &inst));
            }
            Some(literal_union_labels(checker, &t))
        }
        // obj["..."]：对象属性名
        NodeData::IndexedAccessTypeNode(d) if Arc::ptr_eq(&d.index_type, &lit) => {
            let obj_t = checker.get_type_from_type_node(&d.object_type);
            let props = checker.get_apparent_properties(&obj_t);
            Some(props.into_iter().map(|p| p.name.clone()).collect())
        }
        // a === '...'：Go getContextualTypeFromParent——等值比较取另一侧类型
        NodeData::BinaryExpression(d) => {
            if !matches!(
                d.operator_token.kind,
                SyntaxKind::EqualsEqualsEqualsToken
                    | SyntaxKind::EqualsEqualsToken
                    | SyntaxKind::ExclamationEqualsEqualsToken
                    | SyntaxKind::ExclamationEqualsToken
            ) {
                return None;
            }
            let other = if Arc::ptr_eq(&d.left, &lit) {
                Arc::clone(&d.right)
            } else {
                Arc::clone(&d.left)
            };
            let t = checker.get_type_of_node(&other);
            Some(literal_union_labels(checker, &t))
        }
        // var x: 'a'|'b' = '...'：注解类型（Go GetContextualType 默认路径）
        NodeData::VariableDeclaration(d) => {
            let tn = d.type_node.as_ref()?;
            let t = checker.get_type_from_type_node(tn);
            Some(literal_union_labels(checker, &t))
        }
        // case '...'：switch 表达式类型（Go KindCaseClause → getSwitchedType）
        NodeData::CaseOrDefaultClause(_) => {
            let clause = Arc::clone(&parent);
            let Some(switch_stmt) = clause.parent.clone() else {
                return None;
            };
            let Some(case_block) = switch_stmt.parent.clone() else {
                return None;
            };
            let NodeData::SwitchStatement(sw) = &case_block.data else {
                return None;
            };
            let t = checker.get_type_of_node(&sw.expression);
            Some(literal_union_labels(checker, &t))
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
    collect_string_literals(checker, t, &mut out, 0);
    // Go 补全项按 sortText（名称）排序输出
    out.sort();
    out
}

fn collect_string_literals(
    checker: &mut Checker,
    t: &Arc<tsox_checker::checker::types::Type>,
    out: &mut Vec<String>,
    depth: usize,
) {
    use tsox_checker::checker::types::TypeFlags;
    if depth > 8 {
        return;
    }
    if t.flags.intersects(TypeFlags::StringLiteral) {
        if let Some(v) = t.literal_value() {
            if let tsox_checker::checker::types::LiteralValue::String(s) = v {
                out.push(s.clone());
            }
        }
    } else if t.is_union() {
        // 保持联合声明序（@stableTypeOrdering 用例依赖）
        for c in t.types().unwrap_or_default() {
            collect_string_literals(checker, c, out, depth + 1);
        }
    } else if t.flags.intersects(TypeFlags::TypeParameter) {
        // 泛型形参（K extends keyof Foo）：Go GetTypeArgumentConstraint
        if let Some(c) = checker.get_constraint_of_type_parameter(t) {
            collect_string_literals(checker, &c, out, depth + 1);
        }
    }
}
