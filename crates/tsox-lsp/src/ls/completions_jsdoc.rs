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
    ContinuePipeline,
    Blocked,
}

pub fn jsdoc_position_completions(
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

    if let Some(tags) = &d.tags {
        for tag in tags.nodes.iter() {
            if !(tag.pos() <= position && position <= tag.end()) {
                continue;
            }
            // Go getJSDocTagAtPosition：标签名区间（含裸 @ 零宽名）
            if let Some(name) = tag_name_of(tag)
                && name.pos() <= position
                && position <= name.end()
            {
                return JsDocPosition::Labels(
                    JSDOC_TAG_NAMES.iter().map(|s| s.to_string()).collect(),
                );
            }
            // import tag 内（说明符/属性值串）走字符串补全
            if tag.kind == SyntaxKind::JSDocImportTag {
                return JsDocPosition::ImportTag(Arc::clone(tag));
            }
            // 标签的类型表达式内按类型位补全
            if tag_type_expression_span(tag).is_some_and(|(s, e)| s <= position && position <= e) {
                return JsDocPosition::ContinuePipeline;
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
        return JsDocPosition::Labels(
            JSDOC_TAG_NAMES.iter().map(|s| format!("@{s}")).collect(),
        );
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
