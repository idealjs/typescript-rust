#![allow(dead_code)]

use std::sync::Arc;

use tsox_checker::checker::Checker;
use tsox_checker::checker::types::ContextFlags;
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::Node;
use tsox_frontend::ast::SymbolFlags;
use tsox_frontend::ast::SyntaxKind;
use tsox_frontend::ast::SourceFile;
use tsox_frontend::ast::Symbol;

use tsox_frontend::ast::NodeData;

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
    let lit = innermost_string_literal(node, position)?;
    // case ('...') 等括号包裹：字面量的直接父是括号，上跳到语法父
    let mut parent = lit.parent()?;
    while parent.kind == SyntaxKind::ParenthesizedExpression {
        parent = parent.parent()?;
    }
    // '...' as const / '...' as T / <T>'...'：断言不影响补全位语义，
    // 取断言表达式位的上下文型
    if matches!(
        parent.kind,
        SyntaxKind::AsExpression | SyntaxKind::TypeAssertionExpression | SyntaxKind::SatisfiesExpression
    ) {
        let t = checker.get_contextual_type(&parent, ContextFlags::IgnoreNodeInferences);
        return t.map(|t| literal_union_labels(checker, &t));
    }

    match &parent.data {
        // 类型位字面量 `Foo<'...'>` / `Foo<{ x: '...' }>`（Go
        // getStringLiteralCompletionEntries 的 LiteralType 分支 →
        // fromUnionableLiteralType）
        NodeData::LiteralTypeNode(_) => {
            from_unionable_literal_type(checker, &parent, position)
        }
        // f("...")：按实参位取形参约束，对全部重载签名求并集
        // （Go getStringLiteralCompletionsFromSignature ×
        // GetCandidateSignaturesForStringLiteralCompletions）；泛型签名先由
        // 其余实参推断类型参数
        NodeData::CallExpression(d) => {
            let idx = d
                .arguments
                .nodes
                .iter()
                .position(|a| Arc::ptr_eq(a, &lit))?;
            let callee_type = checker.get_type_of_node(&d.expression);
            let structured = callee_type.as_structured()?;
            let sigs = structured.call_signatures();
            let mut labels: Vec<String> = Vec::new();
            for sig in sigs.iter() {
                let sig = sig.clone();
                // Go：无 rest 形参且实参个数超出形参个数的候选剔除
                if !sig.has_rest_parameter() && d.arguments.nodes.len() > sig.parameters.len() {
                    continue;
                }
                let per_sig: Vec<String> = if !sig.type_parameters.is_empty() {
                    let param = sig.parameters.get(idx).cloned()?;
                    let t = checker.get_type_of_symbol(&param);
                    let mut inferred =
                        checker.infer_call_type_arguments(&parent, &sig, &d.arguments.nodes);
                    // 推断值可含未回填的其它类型参数（K=keyof T）：对推断结果
                    // 自身再过一遍替换链闭环
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
                    literal_union_labels(checker, &inst)
                } else {
                    let t = checker.get_type_parameter_at_position(&sig, idx);
                    literal_union_labels(checker, &t)
                };
                labels.extend(per_sig);
            }
            if labels.is_empty() {
                return None;
            }
            labels.sort();
            labels.dedup();
            return Some(labels);
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
        // with { type: "..." }：全局 ImportAttributes 接口同名属性的字面量
        // 并集（Go getContextualImportAttributeType）
        NodeData::ImportAttribute(d) => {
            let attr_name = match &d.name.data {
                NodeData::Identifier(id) => id.text.clone(),
                NodeData::StringLiteral(_) => d.name.text().trim_matches(['"', '\'']).to_string(),
                _ => return None,
            };
            import_attribute_value_labels(checker, &attr_name)
        }
        // {"…"}：对象字面量引号键名位的成员名并集（Go
        // stringLiteralCompletionsForObjectLiteral）；
        // 属性值位 type: '…'：上下文型字面量并集（Go
        // getStringLiteralCompletionsFromContextualType）
        NodeData::PropertyAssignment(_) | NodeData::ShorthandPropertyAssignment(_) => {
            let is_name = match &parent.data {
                NodeData::PropertyAssignment(d) => Arc::ptr_eq(&d.name, &lit),
                NodeData::ShorthandPropertyAssignment(d) => Arc::ptr_eq(&d.name, &lit),
                _ => false,
            };
            if !is_name {
                // IgnoreNodeInferences：正在编辑的实参从推断源剔除，
                // 泛型回落返回位推断（Go completions 探针语义）
                let t = checker.get_contextual_type(&lit, ContextFlags::IgnoreNodeInferences);
                return t.map(|t| literal_union_labels(checker, &t));
            }
            let Some(grand) = parent.parent() else {
                return None;
            };
            if grand.kind != SyntaxKind::ObjectLiteralExpression {
                return None;
            }
            let Some(t) = checker.get_contextual_type(&grand, ContextFlags::None) else {
                return None;
            };
            let completions_type =
                checker.get_contextual_type(&grand, ContextFlags::IgnoreNodeInferences);
            let members =
                super::completions_object_like::properties_for_object_expression(
                    checker,
                    &t,
                    completions_type.as_ref(),
                    &grand,
                );
            Some(members.into_iter().map(|m| m.name.clone()).collect())
        }
        // var x: 'a'|'b' = '...'：注解类型（Go GetContextualType 默认路径）
        NodeData::VariableDeclaration(d) => {
            let tn = d.type_node.as_ref()?;
            let t = checker.get_type_from_type_node(tn);
            Some(literal_union_labels(checker, &t))
        }
        // case '...'：switch 表达式类型（Go KindCaseClause → getSwitchedType），
        // 已用的 case 字面量排除（Go newCaseClauseTracker）
        NodeData::CaseOrDefaultClause(_) => {
            let clause = Arc::clone(&parent);
            let Some(switch_stmt) = clause.parent() else {
                return None;
            };
            let Some(case_block) = switch_stmt.parent() else {
                return None;
            };
            let NodeData::SwitchStatement(sw) = &case_block.data else {
                return None;
            };
            let t = checker.get_type_of_node(&sw.expression);
            let mut labels = literal_union_labels(checker, &t);
            let used = case_block_literals(&sw.case_block, lit.pos());
            labels.retain(|l| !used.iter().any(|u| u == l));
            Some(labels)
        }
        // o["..."]：表达式类型的属性名（Go KindElementAccessExpression →
        // stringLiteralCompletionsFromProperties）
        NodeData::ElementAccessExpression(d) => {
            let arg = skip_parens(Arc::clone(&d.argument_expression));
            if !Arc::ptr_eq(&arg, &lit) {
                return None;
            }
            let t = checker.get_type_of_node(&d.expression);
            let mut names: Vec<String> = checker
                .get_apparent_properties(&t)
                .into_iter()
                .filter(|s| {
                    !s.value_declaration
                        .as_ref()
                        .is_some_and(|decl| decl.kind == SyntaxKind::PrivateIdentifier)
                })
                .map(|s| s.name.clone())
                .collect();
            names.sort();
            Some(names)
        }
        _ => None,
    }
}

/// case block 内已用的字符串字面量（跳过正在编辑的 editing_lit_pos）
fn case_block_literals(block: &Arc<Node>, editing_lit_pos: usize) -> Vec<String> {
    fn collect(expr: &Arc<Node>, editing_lit_pos: usize, out: &mut Vec<String>) {
        if expr.kind == SyntaxKind::StringLiteral && expr.pos() != editing_lit_pos {
            out.push(expr.text().trim_matches(['"', '\'']).to_string());
            return;
        }
        tsox_frontend::ast::node_data_generated::for_each_child(expr, |c| {
            collect(c, editing_lit_pos, out);
            false
        });
    }
    let NodeData::CaseBlock(d) = &block.data else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for clause in d.clauses.iter() {
        if let NodeData::CaseOrDefaultClause(cd) = &clause.data {
            collect(&cd.expression, editing_lit_pos, &mut out);
        }
    }
    out
}

/// Go ast.SkipParentheses：下穿括号取内层表达式
fn skip_parens(mut n: Arc<Node>) -> Arc<Node> {
    while n.kind == SyntaxKind::ParenthesizedExpression {
        let NodeData::ParenthesizedExpression(d) = &n.data else {
            break;
        };
        n = Arc::clone(&d.expression);
    }
    n
}

fn innermost_string_literal(node: &Arc<Node>, position: usize) -> Option<Arc<Node>> {
    let mut cur = Some(Arc::clone(node));
    while let Some(c) = cur {
        if c.kind == tsox_frontend::ast::SyntaxKind::StringLiteral {
            return Some(c);
        }
        if c.pos() > position || c.end() < position {
            return None;
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
    None
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

/// Go fromUnionableLiteralType：类型位字符串字面量的约束来源判定。
/// literal_type 为 LiteralType 节点，grandparent 为其（跨括号）父节点
fn from_unionable_literal_type(
    checker: &mut Checker,
    literal_type: &Arc<Node>,
    position: usize,
) -> Option<Vec<String>> {
    let grandparent = literal_type.parent()?;
    match grandparent.kind {
        SyntaxKind::CallExpression
        | SyntaxKind::NewExpression
        | SyntaxKind::TaggedTemplateExpression
        | SyntaxKind::ExpressionWithTypeArguments
        | SyntaxKind::JsxOpeningElement
        | SyntaxKind::JsxSelfClosingElement
        | SyntaxKind::TypeReference => {
            let mut type_argument = Arc::clone(literal_type);
            while let Some(p) = type_argument.parent() {
                if Arc::ptr_eq(&p, &grandparent) {
                    break;
                }
                type_argument = p;
            }
            let t = checker.get_type_argument_constraint(&type_argument)?;
            Some(literal_union_labels(checker, &t))
        }
        SyntaxKind::IndexedAccessType => {
            let NodeData::IndexedAccessTypeNode(d) = &grandparent.data else {
                return None;
            };
            if !(d.index_type.pos() <= position && position <= d.index_type.end()) {
                return None;
            }
            let obj_t = checker.get_type_from_type_node(&d.object_type);
            Some(
                checker
                    .get_apparent_properties(&obj_t)
                    .into_iter()
                    .map(|p| p.name.clone())
                    .collect(),
            )
        }
        SyntaxKind::UnionType => {
            let result = from_unionable_literal_type(checker, &grandparent, position)?;
            let used = already_used_literals_in_union(&grandparent, literal_type);
            Some(
                result
                    .into_iter()
                    .filter(|l| !used.contains(l))
                    .collect(),
            )
        }
        SyntaxKind::PropertySignature => {
            let t = super::completions_object_like_types::constraint_of_type_argument_property(
                checker, &grandparent,
            )?;
            Some(literal_union_labels(checker, &t))
        }
        _ => None,
    }
}

fn already_used_literals_in_union(union: &Arc<Node>, current: &Arc<Node>) -> Vec<String> {
    let NodeData::UnionTypeNode(d) = &union.data else {
        return Vec::new();
    };
    d.types
        .nodes
        .iter()
        .filter(|t| !Arc::ptr_eq(t, current) && t.kind == SyntaxKind::LiteralType)
        .filter_map(|t| match &t.data {
            NodeData::LiteralTypeNode(l) => Some(l.literal.text().to_string()),
            _ => None,
        })
        .collect()
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

/// Go getStringLiteralCompletionsFromModuleNames 的模块说明符补全：
/// 相对/根路径走程序内文件枚举；非相对（裸包名）走 node_modules 管线
/// （completions_path 的 exports/typesVersions 处理）
pub fn relative_module_specifier_labels(
    service: &LanguageService,
    program: &Arc<tsox_compile::compiler::Program>,
    file: &Arc<SourceFile>,
    node: &Arc<Node>,
    text: &str,
    position: usize,
) -> Option<Vec<String>> {
    let lit = innermost_string_literal(node, position)?;
    if !is_module_specifier_literal(&lit) {
        return None;
    }
    let raw = text.get(lit.pos()..lit.end().min(text.len()))?;
    let content = raw.trim_matches(|c| c == '"' || c == '\'' || c == '`');
    if !(content.starts_with("./")
        || content.starts_with("../")
        || content.starts_with('/'))
    {
        return Some(super::completions_path::non_relative_module_labels(
            service,
            program,
            &file.file_name,
            content,
        ));
    }
    let directory = match content.rfind('/') {
        Some(idx) => &content[..=idx],
        None => return None,
    };

    let script_dir = match file.file_name.rfind('/') {
        Some(idx) => &file.file_name[..idx],
        None => "",
    };
    let base_dir = normalize_path(&format!("{script_dir}/{directory}"));
    // base_dir 归一化到根（`..//` 说明符）时不能再叠尾斜杠
    let with_slash = if base_dir == "/" {
        "/".to_string()
    } else {
        format!("{base_dir}/")
    };
    if !program
        .source_files()
        .iter()
        .any(|f| f.file_name.starts_with(&with_slash))
    {
        return None;
    }

    let mut labels: Vec<String> = Vec::new();
    let prefix = with_slash;
    for other in program.source_files() {
        if other.file_name == file.file_name {
            continue;
        }
        let Some(rest) = other.file_name.strip_prefix(&prefix) else {
            continue;
        };
        if rest.is_empty() {
            continue;
        }
        match rest.find('/') {
            Some(idx) => labels.push(rest[..idx].to_string()),
            None => labels.push(strip_module_extension(rest).to_string()),
        }
    }
    labels.sort();
    labels.dedup();
    Some(labels)
}

fn is_module_specifier_literal(lit: &Arc<Node>) -> bool {
    let Some(parent) = &lit.parent() else {
        return false;
    };
    match &parent.data {
        NodeData::ImportDeclaration(d) => Arc::ptr_eq(&d.module_specifier, lit),
        NodeData::ExportDeclaration(d) => {
            d.module_specifier.as_ref().is_some_and(|s| Arc::ptr_eq(s, lit))
        }
        NodeData::ExternalModuleReference(d) => Arc::ptr_eq(&d.expression, lit),
        NodeData::CallExpression(_) => true,
        _ => false,
    }
}

fn normalize_path(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => continue,
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    format!("/{}", parts.join("/"))
}

fn strip_module_extension(name: &str) -> &str {
    for ext in [".d.ts", ".d.mts", ".d.cts", ".ts", ".tsx", ".mts", ".cts", ".js", ".jsx", ".mjs", ".cjs"] {
        if let Some(stripped) = name.strip_suffix(ext) {
            return stripped;
        }
    }
    name

}

/// with { key: "value" } 值位的字面量并集（Go getContextualImportAttributeType：
/// 全局 ImportAttributes/ImportAssertions 接口同名属性类型）
pub(super) fn import_attribute_value_labels(
    checker: &mut Checker,
    attr_name: &str,
) -> Option<Vec<String>> {
    let global = checker
        .get_global_symbol_by_name("ImportAttributes", SymbolFlags::TYPE)
        .or_else(|| checker.get_global_symbol_by_name("ImportAssertions", SymbolFlags::TYPE))?;
    let t = checker.resolve_interface_type_ex(&global, None);
    let prop = checker.get_property_of_type(&t, attr_name)?;
    let prop_t = checker.get_type_of_symbol(&prop);
    Some(literal_union_labels(checker, &prop_t))
}
