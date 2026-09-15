use std::sync::Arc;

use tsox_frontend::ast::{Node, NodeData, SourceFile, SyntaxKind};
use tsox_frontend::parser::parse_jsdoc_comment_range;

use super::completions_context;

// Go completions.go jsDocTagNames：标签名与带 @ 前缀的完整标签共用
const JSDOC_TAG_NAMES: &[&str] = &[
    "abstract",
    "access",
    "alias",
    "argument",
    "async",
    "augments",
    "author",
    "borrows",
    "callback",
    "class",
    "classdesc",
    "constant",
    "constructor",
    "constructs",
    "copyright",
    "default",
    "deprecated",
    "description",
    "emits",
    "enum",
    "event",
    "example",
    "exports",
    "extends",
    "external",
    "field",
    "file",
    "fileoverview",
    "fires",
    "function",
    "generator",
    "global",
    "hideconstructor",
    "host",
    "ignore",
    "implements",
    "import",
    "inheritdoc",
    "inner",
    "instance",
    "interface",
    "kind",
    "lends",
    "license",
    "link",
    "linkcode",
    "linkplain",
    "listens",
    "member",
    "memberof",
    "method",
    "mixes",
    "module",
    "name",
    "namespace",
    "overload",
    "override",
    "package",
    "param",
    "private",
    "prop",
    "property",
    "protected",
    "public",
    "readonly",
    "requires",
    "returns",
    "satisfies",
    "see",
    "since",
    "static",
    "summary",
    "template",
    "this",
    "throws",
    "todo",
    "tutorial",
    "type",
    "typedef",
    "var",
    "variation",
    "version",
    "virtual",
    "yields",
];

/// Go isInComment 的 JSDoc 通道：标签名编辑位只出标签补全；类型表达式/
/// import tag 内继续常规管线；其余 doc 注释位一律空。None=不在 doc 注释内
pub enum JsDocPosition {
    NotInDocComment,
    Labels(Vec<String>),
    ImportTag(Arc<Node>),
    /// Go completionDataJSDocParameterName：@param 名字位（名 missing 或
    /// 光标在名区间）给未标注的函数形参名
    ParameterNames(Vec<String>),
    /// Go insideJSDocTagTypeExpression：标签类型表达式内的纯类型位
    TypeExpression,
    /// 类型表达式内的点成员位（import("./m"). 等）：携带 JSDocTypeExpression
    /// 子树根，成员补全在 jsdoc 类型树内找接收者
    TypeDotMember(Arc<Node>),
    #[allow(dead_code)]
    ContinuePipeline,
    Blocked,
}

pub fn jsdoc_position_completions(
    checker: &mut tsox_checker::checker::Checker,
    file: &Arc<SourceFile>,
    jsx: bool,
    position: usize,
) -> JsDocPosition {
    let ranges = completions_context::comment_ranges(&file.text, jsx);
    let Some(range) = ranges.iter().find(|r| {
        r.pos < position && position < r.end && completions_context::is_doc_comment(r, &file.text)
    }) else {
        return JsDocPosition::NotInDocComment;
    };
    let Some(doc) = parse_jsdoc_comment_range(file, range.pos, range.end) else {
        return JsDocPosition::Blocked;
    };
    let NodeData::JSDoc(d) = &doc.data else {
        return JsDocPosition::Blocked;
    };

    // Go：position 前一字节是 '@'（tag 名未打）→ 直接给全量标签名 +
    // 智能参数补全（tagNameOnly 语义，剥前导 @）
    if position > 0 && file.text.as_bytes()[position - 1] == b'@' {
        let mut labels: Vec<String> = JSDOC_TAG_NAMES.iter().map(|s| s.to_string()).collect();
        let empty: Vec<Arc<Node>> = Vec::new();
        let tags_slice = d.tags.as_ref().map(|t| t.nodes.as_slice()).unwrap_or(&empty);
        labels.extend(super::completions_jsdoc_params::jsdoc_parameter_completions(
            checker,
            file,
            range.pos,
            range.end,
            tags_slice,
            position,
            true,
        ));
        return JsDocPosition::Labels(labels);
    }

    if let Some(tags) = &d.tags {
        let _n = tags.nodes.len();
        for (idx, tag) in tags.nodes.iter().enumerate() {
            // Go JSDocTag 的 span 含尾随 comment 区（到下一 tag/注释尾）：
            // 间隔归属前一 tag
            let span_end = tags
                .nodes
                .get(idx + 1)
                .map(|next| next.pos())
                .unwrap_or(range.end);
            if !(tag.pos() <= position && position < span_end) {
                continue;
            }
            // Go getJSDocTagAtPosition：标签名区间（含裸 @ 零宽名）
            if let Some(name) = tag_name_of(tag)
                && name.pos() <= position
                && position <= name.end()
            {
                let mut labels: Vec<String> = JSDOC_TAG_NAMES
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                labels.extend(super::completions_jsdoc_params::jsdoc_parameter_completions(
                    checker,
                    file,
                    range.pos,
                    range.end,
                    tags.nodes.as_slice(),
                    position,
                    true,
                ));
                return JsDocPosition::Labels(labels);
            }
            // import tag 内（说明符/属性值串）走字符串补全
            if tag.kind == SyntaxKind::JSDocImportTag {
                return JsDocPosition::ImportTag(Arc::clone(tag));
            }
            // 标签的类型表达式内按类型位补全
            if let Some((s, e)) = tag_type_expression_span(tag)
                && s <= position
                && position <= e
            {
                // 嵌套对象字面量的属性名位（`{ {…` 的内层 { 后）：无上下文
                // 约束（Go ObjectPropertyDeclaration 空集），不回退 scope；
                // 外层开括号后仍是类型位（`{ Foo.…`）
                let mut q = position;
                while q > s
                    && file.text[s..q].chars().last().is_some_and(char::is_whitespace)
                {
                    q -= file.text[s..q].chars().last().unwrap().len_utf8();
                }
                let nested_object_literal = q > s
                    && &file.text[q - 1..q] == "{"
                    && file.text[s..q - 1].contains('{');
                if nested_object_literal {
                    return JsDocPosition::Blocked;
                }
                // 点后是成员补全位（import("./m"). 等）：在 jsdoc 类型树内
                // 解析；其余类型位给全局类型符号 + 类型关键字
                // （Go insideJSDocTagTypeExpression）
                if file.text[s..position].contains('.') {
                    let mut type_expr: Option<Arc<Node>> = None;
                    tsox_frontend::ast::node_data_generated::for_each_child(tag, |c| {
                        if c.kind == SyntaxKind::JSDocTypeExpression && type_expr.is_none() {
                            type_expr = Some(Arc::clone(c));
                        }
                        type_expr.is_some()
                    });
                    if let Some(te) = type_expr {
                        return JsDocPosition::TypeDotMember(te);
                    }
                }
                return JsDocPosition::TypeExpression;
            }
            // Go IsJSDocParameterTag：名字 missing（类型在前、名未打）或光标
            // 在名区间 → 形参名补全
            if tag.kind == SyntaxKind::JSDocParameterTag
                && let NodeData::JSDocParameterOrPropertyTag(pd) = &tag.data
            {
                let name = Arc::clone(&pd.name);
                if name.pos() >= name.end()
                    || (name.pos() <= position && position <= name.end())
                {
                    return JsDocPosition::ParameterNames(parameter_name_completions(
                        file,
                        tag,
                        &name,
                        tags.nodes.as_slice(),
                        range.end,
                    ));
                }
            }
        }
    }

    // Go completionDataJSDocTag：本行 position 之前只有边距字符
    // （空白、*、/、(、)、|）时给带 @ 前缀的完整标签
    let line_start = file.text[..position].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let margin_only = file.text[line_start..position]
        .chars()
        .all(|c| c.is_whitespace() || matches!(c, '*' | '/' | '(' | ')' | '|'));
    if margin_only {
        let mut labels: Vec<String> = JSDOC_TAG_NAMES
            .iter()
            .map(|s| format!("@{s}"))
            .collect();
        let empty: Vec<Arc<Node>> = Vec::new();
        labels.extend(super::completions_jsdoc_params::jsdoc_parameter_completions(
            checker,
            file,
            range.pos,
            range.end,
            d.tags.as_ref().map(|t| t.nodes.as_slice()).unwrap_or(&empty),
            position,
            false,
        ));
        return JsDocPosition::Labels(labels);
    }
    JsDocPosition::Blocked
}

fn tag_type_expression_span(tag: &Arc<Node>) -> Option<(usize, usize)> {
    let mut span: Option<(usize, usize)> = None;
    tsox_frontend::ast::node_data_generated::for_each_child(tag, |c| {
        if c.kind == SyntaxKind::JSDocTypeExpression && span.is_none() {
            span = Some((c.pos(), c.end()));
        }
        false
    });
    span
}

fn tag_name_of(tag: &Arc<Node>) -> Option<Arc<Node>> {
    let mut name: Option<Arc<Node>> = None;
    tsox_frontend::ast::node_data_generated::for_each_child(tag, |c| {
        if c.kind == SyntaxKind::Identifier && name.is_none() {
            name = Some(Arc::clone(c));
        }
        false
    });
    name
}

/// Go getJSDocParameterNameCompletions：函数形参名 - 同 JSDoc 内已被其它
/// @param 标注的名字 - nameThusFar 前缀过滤；解构形参不参与
fn parameter_name_completions(
    file: &Arc<SourceFile>,
    tag: &Arc<Node>,
    editing_name: &Arc<Node>,
    all_tags: &[Arc<Node>],
    doc_end: usize,
) -> Vec<String> {
    let name_thus_far = if editing_name.pos() < editing_name.end() {
        editing_name.text().to_string()
    } else {
        String::new()
    };
    // jsdoc 是函数的 leading trivia（pos 不含注释）：注释 range 已含闭合
    // `*/`，锚点取其后跳过空白的首字符（函数关键字处），再上爬函数
    let close = doc_end;
    let anchor = close + (file.text[close..].len() - file.text[close..].trim_start().len());
    let mut cur = Some(crate::ls::completions_helpers::find_deepest_node(
        &file.node,
        anchor,
    ));
    while let Some(c) = &cur
        && !tsox_frontend::ast::is_function_like_kind(c.kind)
    {
        cur = c.parent();
    }
    let Some(fn_node) = cur else {
        return Vec::new();
    };
    let mut params: Vec<Arc<Node>> = Vec::new();
    tsox_frontend::ast::node_data_generated::for_each_child(&fn_node, |c| {
        if c.kind == SyntaxKind::Parameter {
            params.push(Arc::clone(c));
        }
        false
    });
    params
        .iter()
        .filter_map(|p| {
            let NodeData::ParameterDeclaration(pd) = &p.data else {
                return None;
            };
            if pd.name.kind != SyntaxKind::Identifier {
                return None;
            }
            let name = pd.name.text().to_string();
            let annotated = all_tags.iter().any(|t| {
                !Arc::ptr_eq(t, tag)
                    && matches!(&t.data, NodeData::JSDocParameterOrPropertyTag(td)
                        if td.name.kind == SyntaxKind::Identifier && td.name.text() == name)
            });
            if annotated
                || (!name_thus_far.is_empty() && !name.starts_with(&name_thus_far))
            {
                return None;
            }
            Some(name)
        })
        .collect()
}

/// @import 标签内的字符串补全：说明符位出模块名（Go
/// getStringLiteralCompletionsFromModuleNames，非相对走 node_modules），
/// 属性值位出全局 ImportAttributes 接口同名属性的字面量并集
pub fn jsdoc_import_tag_string_labels(
    service: &super::language_service::LanguageService,
    program: &std::sync::Arc<tsox_compile::compiler::Program>,
    checker: &mut tsox_checker::checker::Checker,
    file: &Arc<SourceFile>,
    tag: &Arc<Node>,
    position: usize,
) -> Option<Vec<String>> {
    let NodeData::JSDocImportTag(d) = &tag.data else {
        return None;
    };
    let spec = &d.module_specifier;
    if spec.kind == SyntaxKind::StringLiteral && spec.pos() < position && position < spec.end() {
        let content = spec.text().trim_matches(['"', '\'']);
        if content.starts_with("./") || content.starts_with("../") || content.starts_with('/') {
            return None;
        }
        return Some(super::completions_path::non_relative_module_labels(
            service,
            program,
            &file.file_name,
            content,
        ));
    }
    let attrs = &d.attributes.as_ref()?.data;
    let NodeData::ImportAttributes(ad) = attrs else {
        return None;
    };
    for attr in ad.attributes.nodes.iter() {
        let NodeData::ImportAttribute(at) = &attr.data else {
            continue;
        };
        if at.value.kind == SyntaxKind::StringLiteral
            && at.value.pos() < position
            && position < at.value.end()
        {
            let name = match &at.name.data {
                NodeData::Identifier(id) => id.text.clone(),
                NodeData::StringLiteral(_) => at.name.text().trim_matches(['"', '\'']).to_string(),
                _ => continue,
            };
            return super::string_completions::import_attribute_value_labels(checker, &name);
        }
    }
    None
}
