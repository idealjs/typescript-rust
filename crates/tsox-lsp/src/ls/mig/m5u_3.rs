#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_checker::checker::Checker;
use tsox_frontend::ast::{self, Node, NodeData, NodeList, SourceFile, Symbol, SyntaxKind};
use tsox_frontend::astnav;

// ============ jsdoc.go ============

pub fn declaration_jsdoc_tags(node: &Arc<Node>, file: &Arc<SourceFile>) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("declaration_jsdoc_tags"); 
    if !node.flags.contains(ast::NodeFlags::JSDoc) {
        let mut current = Some(Arc::clone(node));
        while let Some(cur) = current {
            let jsdocs = cur.jsdoc(file);
            if jsdocs.is_empty() {
                current = ast::get_next_jsdoc_comment_location(&cur);
                continue;
            }
            let last_jsdoc = &jsdocs[jsdocs.len() - 1];
            if let Some(tags) = jsdoc_tags(last_jsdoc) {
                return tags;
            }
            current = ast::get_next_jsdoc_comment_location(&cur);
        }
    }
    Vec::new()
}

pub fn jsdoc_tags(jsdoc: &Arc<Node>) -> Option<Vec<Arc<Node>>> { ::tsox_core::fntrace::enter("jsdoc_tags"); 
    match &jsdoc.data {
        NodeData::JSDoc(d) => d.tags.as_ref().map(|tags| tags.nodes.clone()),
        _ => None,
    }
}

pub fn jsdoc_comments(jsdoc: &Arc<Node>) -> Option<Vec<Arc<Node>>> { ::tsox_core::fntrace::enter("jsdoc_comments"); 
    match &jsdoc.data {
        NodeData::JSDoc(d) => Some(d.comment.nodes.clone()),
        _ => None,
    }
}

pub fn get_jsdoc_tag_text(tag: &Arc<Node>) -> String { ::tsox_core::fntrace::enter("get_jsdoc_tag_text"); 
    let comment = scanner_get_text_of_jsdoc_comment(jsdoc_comment_list(tag));
    let add_comment = |s: String| -> String {
        if comment.is_empty() {
            s
        } else {
            format!("{s} {comment}")
        }
    };
    match tag.kind {
        SyntaxKind::JSDocThrowsTag => match &tag.data {
            NodeData::JSDocThrowsTag(d) => match &d.type_expression {
                Some(te) => add_comment(scanner_get_text_of_node(te)),
                None => comment.clone(),
            },
            _ => comment.clone(),
        },
        SyntaxKind::JSDocImplementsTag => match &tag.data {
            NodeData::JSDocImplementsTag(d) => add_comment(scanner_get_text_of_node(&d.class_name)),
            _ => comment.clone(),
        },
        SyntaxKind::JSDocAugmentsTag => match &tag.data {
            NodeData::JSDocAugmentsTag(d) => add_comment(scanner_get_text_of_node(&d.class_name)),
            _ => comment.clone(),
        },
        SyntaxKind::JSDocTemplateTag => {
            let mut b = String::new();
            if let NodeData::JSDocTemplateTag(d) = &tag.data {
                b.push_str(&scanner_get_text_of_node(&d.constraint));
                for (i, tp) in d.type_parameters.nodes.iter().enumerate() {
                        if i == 0 && !b.is_empty() {
                            b.push(' ');
                        }
                        if i != 0 {
                            b.push_str(", ");
                        }
                        b.push_str(&scanner_get_text_of_node(tp));
                }
            }
            if !comment.is_empty() {
                if !b.is_empty() {
                    b.push(' ');
                }
                b.push_str(&comment);
            }
            b
        }
        SyntaxKind::JSDocTypeTag => match &tag.data {
            NodeData::JSDocTypeTag(d) => {
                add_comment(scanner_get_text_of_node(&d.type_expression))
            }
            _ => comment.clone(),
        },
        SyntaxKind::JSDocSatisfiesTag => match &tag.data {
            NodeData::JSDocSatisfiesTag(d) => {
                add_comment(scanner_get_text_of_node(&d.type_expression))
            }
            _ => comment.clone(),
        },
        SyntaxKind::JSDocSeeTag => match &tag.data {
            NodeData::JSDocSeeTag(d) => {
                add_comment(scanner_get_text_of_node(&d.name_expression))
            }
            _ => comment.clone(),
        },
        SyntaxKind::JSDocParameterTag | SyntaxKind::JSDocPropertyTag => {
            if let Some(name) = tag.name() {
                return add_comment(scanner_get_text_of_node(name));
            }
            comment.clone()
        }
        _ => comment.clone(),
    }
}

pub fn jsdoc_comment_list<'a>(tag: &'a Arc<Node>) -> Option<&'a NodeList> { ::tsox_core::fntrace::enter("jsdoc_comment_list"); 
    let comment = match &tag.data {
        NodeData::JSDoc(d) => return Some(&d.comment),
        NodeData::JSDocTypeTag(d) => &d.comment,
        NodeData::JSDocUnknownTag(d) => &d.comment,
        NodeData::JSDocTemplateTag(d) => &d.comment,
        NodeData::JSDocReturnTag(d) => &d.comment,
        NodeData::JSDocPublicTag(d) => &d.comment,
        NodeData::JSDocPrivateTag(d) => &d.comment,
        NodeData::JSDocProtectedTag(d) => &d.comment,
        NodeData::JSDocReadonlyTag(d) => &d.comment,
        NodeData::JSDocOverrideTag(d) => &d.comment,
        NodeData::JSDocDeprecatedTag(d) => &d.comment,
        NodeData::JSDocSeeTag(d) => &d.comment,
        NodeData::JSDocImplementsTag(d) => &d.comment,
        NodeData::JSDocAugmentsTag(d) => &d.comment,
        NodeData::JSDocSatisfiesTag(d) => &d.comment,
        NodeData::JSDocThrowsTag(d) => &d.comment,
        NodeData::JSDocThisTag(d) => &d.comment,
        NodeData::JSDocImportTag(d) => &d.comment,
        NodeData::JSDocCallbackTag(d) => &d.comment,
        NodeData::JSDocOverloadTag(d) => &d.comment,
        NodeData::JSDocTypedefTag(d) => &d.comment,
        NodeData::JSDocParameterOrPropertyTag(d) => &d.comment,
        _ => return None,
    };
    comment.as_deref()
}

pub fn scanner_get_text_of_jsdoc_comment(comments: Option<&NodeList>) -> String { ::tsox_core::fntrace::enter("scanner_get_text_of_jsdoc_comment"); 
    tsox_frontend::scanner::mig::m3i::get_text_of_jsdoc_comment(comments)
}

pub fn scanner_get_text_of_node(node: &Arc<Node>) -> String { ::tsox_core::fntrace::enter("scanner_get_text_of_node"); 
    tsox_frontend::scanner::mig::m3i::get_text_of_node(node)
}

pub fn get_matching_jsdoc_tag(
    c: &Checker,
    node: &Arc<Node>,
    name: &str,
    matches: impl Fn(&Arc<Node>, &str) -> bool,
    seen_symbols: &mut std::collections::HashSet<u64>,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_matching_jsdoc_tag"); 
    let jsdoc = crate::ls::jsdoc::get_jsdoc_or_tag(c, node)?;
    if jsdoc.kind == SyntaxKind::JSDoc {
        if let Some(tags) = jsdoc_tags(&jsdoc) {
            for tag in tags {
                if matches(&tag, name) {
                    return Some(tag);
                }
            }
        }
    }
    None
}

pub fn get_jsdoc_parameter_tag_by_position(
    c: &Checker,
    param: &Arc<Node>,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_jsdoc_parameter_tag_by_position"); 
    let parent = param.parent()?;

    let params = crate::ls::mig::m5u_2::node_parameters(&parent);
    let mut param_index: i64 = -1;
    for (i, p) in params.iter().enumerate() {
        if p.id() == param.id() {
            param_index = i as i64;
            break;
        }
    }
    if param_index < 0 {
        return None;
    }

    let jsdoc = crate::ls::jsdoc::get_jsdoc_or_tag(c, &parent)?;
    if jsdoc.kind != SyntaxKind::JSDoc {
        return None;
    }

    let tags = jsdoc_tags(&jsdoc)?;

    let mut param_tag_index: i64 = 0;
    for tag in tags {
        if tag.kind == SyntaxKind::JSDocParameterTag {
            if param_tag_index == param_index {
                return Some(tag);
            }
            param_tag_index += 1;
        }
    }
    None
}

pub fn is_matching_parameter_tag(tag: &Arc<Node>, name: &str) -> bool { ::tsox_core::fntrace::enter("is_matching_parameter_tag"); 
    tag.kind == SyntaxKind::JSDocParameterTag && is_node_with_name(tag, name)
}

pub fn is_matching_template_tag(tag: &Arc<Node>, name: &str) -> bool { ::tsox_core::fntrace::enter("is_matching_template_tag"); 
    tag.kind == SyntaxKind::JSDocTemplateTag
        && tag_type_parameters(tag)
            .iter()
            .any(|tp| is_node_with_name(tp, name))
}

pub fn tag_type_parameters(tag: &Arc<Node>) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("tag_type_parameters"); 
    match &tag.data {
        NodeData::JSDocTemplateTag(d) => d.type_parameters.nodes.clone(),
        _ => Vec::new(),
    }
}

pub fn is_node_with_name(node: &Arc<Node>, name: &str) -> bool { ::tsox_core::fntrace::enter("is_node_with_name"); 
    node.name()
        .map_or(false, |node_name| {
            ast::is_identifier(&node_name) && node_name.text() == name
        })
}

pub fn no_mapped_location(
    _file: &Arc<SourceFile>,
    _range: tsox_core::core::text::TextRange,
) -> (crate::lsp::lsproto_lsp::Location, u32) { ::tsox_core::fntrace::enter("no_mapped_location"); 
    (
        Default::default(),
        crate::mig::m5u_conv::SPANMAP_FIDELITY_NONE,
    )
}

// ============ jsdoc_snippet.go ============

pub struct DocCommentTemplate {
    pub new_text: String,
}

pub struct CommentOwnerInfo {
    pub comment_owner: Arc<Node>,
    pub parameters: Vec<Arc<Node>>,
    pub has_return: bool,
}

pub fn get_comment_owner_info(
    token_at_pos: &Arc<Node>,
    generate_return_in_doc_template: bool,
) -> Option<CommentOwnerInfo> { ::tsox_core::fntrace::enter("get_comment_owner_info"); 
    let mut node = Some(Arc::clone(token_at_pos));
    while let Some(cur) = node {
        match get_comment_owner_info_worker(&cur, generate_return_in_doc_template) {
            (Some(info), _) => return Some(info),
            (None, true) => return None,
            (None, false) => {}
        }
        node = cur.parent();
    }
    None
}

pub fn get_comment_owner_info_worker(
    comment_owner: &Arc<Node>,
    generate_return_in_doc_template: bool,
) -> (Option<CommentOwnerInfo>, bool) { ::tsox_core::fntrace::enter("get_comment_owner_info_worker"); 
    match comment_owner.kind {
        SyntaxKind::FunctionDeclaration
        | SyntaxKind::FunctionExpression
        | SyntaxKind::MethodDeclaration
        | SyntaxKind::Constructor
        | SyntaxKind::MethodSignature
        | SyntaxKind::ArrowFunction => {
            let parameters = crate::ls::mig::m5u_2::node_parameters(comment_owner);
            (
                Some(CommentOwnerInfo {
                    comment_owner: Arc::clone(comment_owner),
                    has_return: has_return(
                        comment_owner,
                        generate_return_in_doc_template,
                    ),
                    parameters,
                }),
                false,
            )
        }
        SyntaxKind::PropertyAssignment => match &comment_owner.data {
            NodeData::PropertyAssignment(d) => {
                get_comment_owner_info_worker(&d.initializer, generate_return_in_doc_template)
            }
            _ => (None, false),
        },
        SyntaxKind::ClassDeclaration
        | SyntaxKind::InterfaceDeclaration
        | SyntaxKind::EnumDeclaration
        | SyntaxKind::EnumMember
        | SyntaxKind::TypeAliasDeclaration => (
            Some(CommentOwnerInfo {
                comment_owner: Arc::clone(comment_owner),
                parameters: Vec::new(),
                has_return: false,
            }),
            false,
        ),
        SyntaxKind::PropertySignature => {
            let type_node = comment_owner.type_node();
            match type_node {
                Some(type_node) if ast::is_function_type_node(&type_node) => (
                    Some(CommentOwnerInfo {
                        comment_owner: Arc::clone(comment_owner),
                        parameters: crate::ls::mig::m5u_2::node_parameters(&type_node),
                        has_return: has_return(&type_node, generate_return_in_doc_template),
                    }),
                    false,
                ),
                _ => (
                    Some(CommentOwnerInfo {
                        comment_owner: Arc::clone(comment_owner),
                        parameters: Vec::new(),
                        has_return: false,
                    }),
                    false,
                ),
            }
        }
        SyntaxKind::VariableStatement => {
            let declarations = match &comment_owner.data {
                NodeData::VariableStatement(d) => {
                    crate::ls::mig::m5u_2::node_elements(&d.declaration_list)
                }
                _ => Vec::new(),
            };
            if declarations.len() == 1 {
                if let Some(initializer) = declarations[0].initializer() {
                    if let Some(host) = get_right_hand_side_of_assignment(Some(&initializer)) {
                        return (
                            Some(CommentOwnerInfo {
                                comment_owner: Arc::clone(comment_owner),
                                parameters: crate::ls::mig::m5u_2::node_parameters(&host),
                                has_return: has_return(
                                    &host,
                                    generate_return_in_doc_template,
                                ),
                            }),
                            false,
                        );
                    }
                }
            }
            (
                Some(CommentOwnerInfo {
                    comment_owner: Arc::clone(comment_owner),
                    parameters: Vec::new(),
                    has_return: false,
                }),
                false,
            )
        }
        SyntaxKind::SourceFile => (None, true),
        SyntaxKind::ModuleDeclaration => {
            let parent_is_module = comment_owner
                .parent()
                .map_or(false, |p| p.kind == SyntaxKind::ModuleDeclaration);
            if parent_is_module {
                (None, false)
            } else {
                (
                    Some(CommentOwnerInfo {
                        comment_owner: Arc::clone(comment_owner),
                        parameters: Vec::new(),
                        has_return: false,
                    }),
                    false,
                )
            }
        }
        SyntaxKind::ExpressionStatement => match &comment_owner.data {
            NodeData::ExpressionStatement(d) => {
                get_comment_owner_info_worker(&d.expression, generate_return_in_doc_template)
            }
            _ => (None, false),
        },
        SyntaxKind::BinaryExpression => {
            if ast::mig::m3e_4::get_assignment_declaration_kind(comment_owner)
                == ast::mig::m3e_4::JsDeclarationKind::None
            {
                return (None, true);
            }
            if let Some(right) = binary_expression_right(comment_owner) {
                if ast::is_function_like(&right) {
                    return (
                        Some(CommentOwnerInfo {
                            comment_owner: Arc::clone(comment_owner),
                            parameters: crate::ls::mig::m5u_2::node_parameters(&right),
                            has_return: has_return(&right, generate_return_in_doc_template),
                        }),
                        false,
                    );
                }
            }
            (
                Some(CommentOwnerInfo {
                    comment_owner: Arc::clone(comment_owner),
                    parameters: Vec::new(),
                    has_return: false,
                }),
                false,
            )
        }
        SyntaxKind::PropertyDeclaration => {
            if let Some(initializer) = comment_owner.initializer() {
                if ast::is_function_expression_or_arrow_function(&initializer) {
                    return (
                        Some(CommentOwnerInfo {
                            comment_owner: Arc::clone(comment_owner),
                            parameters: crate::ls::mig::m5u_2::node_parameters(&initializer),
                            has_return: has_return(
                                &initializer,
                                generate_return_in_doc_template,
                            ),
                        }),
                        false,
                    );
                }
            }
            (None, false)
        }
        _ => (None, false),
    }
}

pub fn binary_expression_right(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("binary_expression_right"); 
    match &node.data {
        NodeData::BinaryExpression(d) => Some(Arc::clone(&d.right)),
        _ => None,
    }
}

pub fn has_return(node: &Arc<Node>, generate_return_in_doc_template: bool) -> bool { ::tsox_core::fntrace::enter("has_return"); 
    if !generate_return_in_doc_template {
        return false;
    }
    if ast::is_function_type_node(node) {
        return true;
    }
    if ast::is_arrow_function(node) {
        if let Some(body) = node.body() {
            if ast::is_expression(&body) {
                return true;
            }
        }
    }
    ast::is_function_like_declaration(node)
        && node.body().map_or(false, |body| {
            ast::is_block(&body)
                && ast::mig::m3e_4::for_each_return_statement(&body, |_| true)
        })
}

pub fn get_right_hand_side_of_assignment(right_hand_side: Option<&Arc<Node>>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_right_hand_side_of_assignment"); 
    let mut right_hand_side = Arc::clone(right_hand_side?);
    while right_hand_side.kind == SyntaxKind::ParenthesizedExpression {
        match &right_hand_side.data {
            NodeData::ParenthesizedExpression(d) => {
                right_hand_side = Arc::clone(&d.expression);
            }
            _ => return None,
        }
    }
    match right_hand_side.kind {
        SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction => {
            Some(Arc::clone(&right_hand_side))
        }
        SyntaxKind::ClassExpression => crate::ls::mig::m5u_2::node_members(&right_hand_side)
            .into_iter()
            .find(|member| ast::is_constructor_declaration(member)),
        _ => None,
    }
}

pub fn parameter_doc_comments(
    parameters: &[Arc<Node>],
    is_java_script_file: bool,
    indentation: &str,
    new_line: &str,
) -> String { ::tsox_core::fntrace::enter("parameter_doc_comments"); 
    let mut b = String::new();
    for (i, parameter) in parameters.iter().enumerate() {
        let mut param_name = format!("param{i}");
        if let Some(name) = parameter.name() {
            if ast::is_identifier(name) {
                param_name = name.text().to_string();
            }
        }
        let param_type = if is_java_script_file {
            if parameter_dot_dot_dot_token(parameter).is_some() {
                "{...any} "
            } else {
                "{any} "
            }
        } else {
            ""
        };
        b.push_str(indentation);
        b.push_str(" * @param ");
        b.push_str(param_type);
        b.push_str(&param_name);
        b.push_str(new_line);
    }
    b
}

pub fn parameter_dot_dot_dot_token(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("parameter_dot_dot_dot_token"); 
    match &node.data {
        NodeData::ParameterDeclaration(d) => d.dot_dot_dot_token.as_ref().map(Arc::clone),
        _ => None,
    }
}

pub fn returns_doc_comment(indentation: &str, new_line: &str) -> String { ::tsox_core::fntrace::enter("returns_doc_comment"); 
    format!("{indentation} * @returns{new_line}")
}

pub fn get_indentation_string_at_position(
    source_file: &Arc<SourceFile>,
    position: usize,
) -> String { ::tsox_core::fntrace::enter("get_indentation_string_at_position"); 
    let text = &source_file.text;
    let line_start = get_line_start_position_for_position(position, source_file);
    let mut pos = line_start;
    while pos < position {
        let Some(ch) = text[pos..].chars().next() else {
            break;
        };
        if !is_white_space_single_line(ch) {
            break;
        }
        pos += ch.len_utf8();
    }
    text[line_start..pos].to_string()
}

pub fn is_white_space_single_line(ch: char) -> bool { ::tsox_core::fntrace::enter("is_white_space_single_line"); 
    tsox_core::stringutil::is_white_space_single_line(ch)
}

pub fn is_white_space_like(ch: char) -> bool { ::tsox_core::fntrace::enter("is_white_space_like"); 
    tsox_core::stringutil::is_white_space_like(ch)
}

pub fn get_doc_comment_end_at_position(
    file: &Arc<SourceFile>,
    position: usize,
) -> (usize, bool, bool) { ::tsox_core::fntrace::enter("get_doc_comment_end_at_position"); 
    let text = &file.text;
    let line_start = get_line_start_position_for_position(position, file);
    let line_end = get_line_end_of_position(file, position);
    let prefix = &text[line_start..position];
    let suffix = &text[position..line_end];
    if !trim_right_single_line_whitespace(prefix).ends_with("/**") {
        return (0, false, false);
    }
    let (suffix_end, has_closing) = get_jsdoc_snippet_suffix_end(suffix);
    (position + suffix_end, true, has_closing)
}

pub fn skip_whitespace(text: &str, mut position: usize) -> usize { ::tsox_core::fntrace::enter("skip_whitespace"); 
    while position < text.len() {
        let Some(ch) = text[position..].chars().next() else {
            break;
        };
        if !is_white_space_like(ch) {
            break;
        }
        position += ch.len_utf8();
    }
    position
}

pub fn is_non_empty_jsdoc(jsdoc: Option<&Arc<Node>>) -> bool { ::tsox_core::fntrace::enter("is_non_empty_jsdoc"); 
    let Some(jsdoc) = jsdoc else {
        return false;
    };
    match &jsdoc.data {
        NodeData::JSDoc(d) => {
            !d.comment.nodes.is_empty()
                || d.tags.as_ref().map_or(false, |t| !t.nodes.is_empty())
        }
        _ => false,
    }
}

pub fn has_jsdoc_tags(node: &Arc<Node>, file: &Arc<SourceFile>) -> bool { ::tsox_core::fntrace::enter("has_jsdoc_tags"); 
    let jsdocs = node.jsdoc(file);
    if jsdocs.is_empty() {
        return false;
    }
    let last = &jsdocs[jsdocs.len() - 1];
    jsdoc_tags(last).map_or(false, |tags| !tags.is_empty())
}

pub fn strip_jsdoc_template_indentation(template: &str, new_line: &str) -> String { ::tsox_core::fntrace::enter("strip_jsdoc_template_indentation"); 
    let lines: Vec<String> = template
        .split(new_line)
        .map(|line| {
            let trimmed = line.trim_start_matches([' ', '\t']);
            if trimmed.starts_with('/') {
                trimmed.to_string()
            } else if trimmed.starts_with('*') {
                format!(" {trimmed}")
            } else {
                line.to_string()
            }
        })
        .collect();
    lines.join(new_line)
}

pub fn transform_jsdoc_template_lines(
    template: &str,
    new_line: &str,
    snippet_index: &mut i32,
) -> String { ::tsox_core::fntrace::enter("transform_jsdoc_template_lines"); 
    let mut lines: Vec<String> = template.split(new_line).map(str::to_string).collect();
    for i in 0..lines.len() {
        let line = lines[i].clone();
        if i > 0 && lines[i - 1].starts_with("/**") && line_has_only_jsdoc_asterisk(&line) {
            lines[i] = format!("{line}$0");
            continue;
        }
        if let Some(transformed) = transform_jsdoc_param_line(&line, snippet_index) {
            lines[i] = transformed;
            continue;
        }
        if let Some(transformed) = transform_jsdoc_returns_line(&line, snippet_index) {
            lines[i] = transformed;
        }
    }
    lines.join(new_line)
}

pub fn line_has_only_jsdoc_asterisk(line: &str) -> bool { ::tsox_core::fntrace::enter("line_has_only_jsdoc_asterisk"); 
    let line = line.trim_start_matches([' ', '\t']);
    line.starts_with('*') && is_only_spaces_or_tabs(&line[1..])
}

pub fn transform_jsdoc_param_line(line: &str, snippet_index: &mut i32) -> Option<String> { ::tsox_core::fntrace::enter("transform_jsdoc_param_line"); 
    let mut prefix = "";
    let mut rest = line;
    if rest.starts_with(' ') {
        prefix = " ";
        rest = &rest[1..];
    }
    if !rest.starts_with("* @param") {
        return None;
    }
    rest = &rest["* @param".len()..];
    if !starts_with_single_line_whitespace(rest) {
        return None;
    }
    rest = rest.trim_start_matches([' ', '\t']);

    let mut type_text = String::new();
    if rest.starts_with('{') {
        let close_brace = rest.find('}')?;
        type_text = format!(" {}", &rest[..close_brace + 1]);
        rest = &rest[close_brace + 1..];
        if !starts_with_single_line_whitespace(rest) {
            return None;
        }
        rest = rest.trim_start_matches([' ', '\t']);
    }

    let (param_name, rest) = scan_non_whitespace(rest)?;
    if !is_only_spaces_or_tabs(rest) {
        return None;
    }

    let mut out = format!("{prefix}* @param ");
    if type_text == " {any}" || type_text == " {*}" {
        out.push_str(&format!("{{${{{}:*}}}} ", snippet_index));
        *snippet_index += 1;
    } else if !type_text.is_empty() {
        out.push_str(&format!("{type_text} "));
    }
    out.push_str(&format!("{param_name} ${{{}}}", snippet_index));
    *snippet_index += 1;
    Some(out)
}

pub fn transform_jsdoc_returns_line(line: &str, snippet_index: &mut i32) -> Option<String> { ::tsox_core::fntrace::enter("transform_jsdoc_returns_line"); 
    let mut prefix = "";
    let mut rest = line;
    if rest.starts_with(' ') {
        prefix = " ";
        rest = &rest[1..];
    }
    if !rest.starts_with("* @returns") || !is_only_spaces_or_tabs(&rest["* @returns".len()..]) {
        return None;
    }
    let text = format!("{prefix}* @returns ${{{}}}", snippet_index);
    *snippet_index += 1;
    Some(text)
}

pub fn scan_non_whitespace(text: &str) -> Option<(String, &str)> { ::tsox_core::fntrace::enter("scan_non_whitespace"); 
    if text.is_empty() {
        return None;
    }
    let mut i = 0;
    while i < text.len() {
        let Some(ch) = text[i..].chars().next() else {
            return None;
        };
        if is_white_space_like(ch) {
            if i == 0 {
                return None;
            }
            return Some((text[..i].to_string(), &text[i..]));
        }
        i += ch.len_utf8();
    }
    Some((text.to_string(), ""))
}

pub fn is_jsdoc_snippet_prefix(prefix: &str) -> bool { ::tsox_core::fntrace::enter("is_jsdoc_snippet_prefix"); 
    let trimmed = trim_right_single_line_whitespace(prefix);
    if trimmed.ends_with("/**") {
        return true;
    }
    let start = skip_single_line_whitespace(prefix, 0);
    if start >= trimmed.len() || !trimmed[start..].starts_with('/') {
        return false;
    }
    if start + 3 > trimmed.len() {
        return false;
    }
    for b in trimmed[start + 1..].bytes() {
        if b != b'*' {
            return false;
        }
    }
    trimmed.len() - start >= 3
}

pub fn get_jsdoc_snippet_prefix_start(prefix: &str) -> Option<usize> { ::tsox_core::fntrace::enter("get_jsdoc_snippet_prefix_start"); 
    let trimmed = trim_right_single_line_whitespace(prefix);
    let bytes = trimmed.as_bytes();
    let mut i = trimmed.len() as i64 - 1;
    while i >= 0 && bytes[i as usize] == b'*' {
        if i > 0 && bytes[(i - 1) as usize] == b'/' {
            return Some((i - 1) as usize);
        }
        i -= 1;
    }
    if trimmed.ends_with('/') {
        return Some(trimmed.len() - 1);
    }
    None
}

pub fn is_jsdoc_snippet_suffix(suffix: &str) -> bool { ::tsox_core::fntrace::enter("is_jsdoc_snippet_suffix"); 
    let start = skip_single_line_whitespace(suffix, 0);
    let trimmed = trim_right_single_line_whitespace(&suffix[start..]);
    if trimmed.is_empty() {
        return true;
    }
    if !trimmed.ends_with('/') {
        return false;
    }
    for b in &trimmed.as_bytes()[..trimmed.len() - 1] {
        if *b != b'*' {
            return false;
        }
    }
    true
}

pub fn get_jsdoc_snippet_suffix_end(suffix: &str) -> (usize, bool) { ::tsox_core::fntrace::enter("get_jsdoc_snippet_suffix_end"); 
    let mut pos = skip_single_line_whitespace(suffix, 0);
    let bytes = suffix.as_bytes();
    while pos < suffix.len() && bytes[pos] == b'*' {
        pos += 1;
    }
    if pos < suffix.len() && bytes[pos] == b'/' {
        return (pos + 1, true);
    }
    (0, false)
}

pub fn trim_right_single_line_whitespace(text: &str) -> String { ::tsox_core::fntrace::enter("trim_right_single_line_whitespace"); 
    let mut end = 0;
    let mut pos = 0;
    while pos < text.len() {
        let Some(ch) = text[pos..].chars().next() else {
            break;
        };
        pos += ch.len_utf8();
        if !is_white_space_single_line(ch) {
            end = pos;
        }
    }
    text[..end].to_string()
}

pub fn skip_single_line_whitespace(text: &str, mut pos: usize) -> usize { ::tsox_core::fntrace::enter("skip_single_line_whitespace"); 
    while pos < text.len() {
        let Some(ch) = text[pos..].chars().next() else {
            break;
        };
        if !is_white_space_single_line(ch) {
            break;
        }
        pos += ch.len_utf8();
    }
    pos
}

pub fn is_only_single_line_whitespace(text: &str) -> bool { ::tsox_core::fntrace::enter("is_only_single_line_whitespace"); 
    skip_single_line_whitespace(text, 0) == text.len()
}

pub fn starts_with_single_line_whitespace(text: &str) -> bool { ::tsox_core::fntrace::enter("starts_with_single_line_whitespace"); 
    if text.is_empty() {
        return false;
    }
    text.chars()
        .next()
        .map_or(false, is_white_space_single_line)
}

pub fn is_only_spaces_or_tabs(text: &str) -> bool { ::tsox_core::fntrace::enter("is_only_spaces_or_tabs"); 
    text.bytes().all(|b| b == b' ' || b == b'\t')
}

pub fn get_line_start_position_for_position(position: usize, file: &Arc<SourceFile>) -> usize { ::tsox_core::fntrace::enter("get_line_start_position_for_position"); 
    let line = file.line_map.line_at(position);
    match file.line_map.line_starts.get(line) {
        Some(&start) => start as usize,
        None => 0,
    }
}

pub fn get_line_end_of_position(file: &Arc<SourceFile>, position: usize) -> usize { ::tsox_core::fntrace::enter("get_line_end_of_position"); 
    crate::ls::mig::m5r::get_line_end_of_position(file, position)
}

impl crate::ls::language_service::LanguageService {
    pub fn get_jsdoc_snippet_completion_range(
        &self,
        file: &Arc<SourceFile>,
        position: usize,
        new_text: &str,
    ) -> Option<crate::ls::mig::m5q2b_3::lsproto::TextEditOrInsertReplaceEdit> { ::tsox_core::fntrace::enter("get_jsdoc_snippet_completion_range"); 
        let text = &file.text;
        let line_start = get_line_start_position_for_position(position, file);
        let prefix = &text[line_start..position];
        let mut start = position;
        if let Some(prefix_start) = get_jsdoc_snippet_prefix_start(prefix) {
            start = line_start + prefix_start;
        }

        let line_end = get_line_end_of_position(file, position);
        let suffix = &text[position..line_end];
        let mut end = position;
        if let (suffix_end, true) = get_jsdoc_snippet_suffix_end(suffix) {
            end += suffix_end;
        }

        let script_view = crate::mig::m5u_conv::SourceFileScriptView { file: Arc::clone(file) };
        let converters = crate::mig::m5u_conv::new_converters(
            crate::ls::lsconv_converters::PositionEncodingKind::Utf16,
            Box::new(crate::ls::lsconv_linemap::compute_lsp_line_starts),
        );
        let (replacement_range, fidelity) =
            converters.to_lsp_range(&script_view, tsox_core::core::text::TextRange::new(start, end));
        if !fidelity_is_exact(&fidelity) {
            return None;
        }
        if client_supports_item_insert_replace() {
            return Some(crate::ls::mig::m5q2b_3::lsproto::TextEditOrInsertReplaceEdit {
                insert_replace_edit: Some(crate::ls::mig::m5q2b_3::lsproto::InsertReplaceEdit {
                    new_text: new_text.to_string(),
                    insert: replacement_range.clone(),
                    replace: replacement_range,
                }),
                text_edit: None,
            });
        }
        Some(crate::ls::mig::m5q2b_3::lsproto::TextEditOrInsertReplaceEdit {
            insert_replace_edit: None,
            text_edit: Some(crate::lsp::lsproto_lsp::TextEdit {
                new_text: new_text.to_string(),
                range: replacement_range,
            }),
        })
    }
}

pub fn fidelity_is_exact(fidelity: &u32) -> bool { ::tsox_core::fntrace::enter("fidelity_is_exact"); 
    *fidelity == crate::mig::m5u_conv::SPANMAP_FIDELITY_EXACT
}

pub fn client_supports_item_insert_replace() -> bool { ::tsox_core::fntrace::enter("client_supports_item_insert_replace"); 
    crate::ls::mig::m5q_3::client_supports_item_insert_replace(
        &crate::mig::m5m::ResolvedClientCapabilitiesContext { capabilities: None },
    )
}
