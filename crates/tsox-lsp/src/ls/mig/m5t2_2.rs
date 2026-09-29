#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use crate::ls::language_service::LanguageService;
use crate::ls::lsutil_symbol_display::{ScriptElementKind, ScriptElementKindModifier};
use crate::lsp::lsproto_lsp_basic::{Location, Range};
use tsox_checker::checker::Checker;
use tsox_checker::checker::nodebuilder_type_format_flags_2::TypeFormatFlags;
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::mig::m3e_2::is_import_meta;
use tsox_frontend::ast::mig::m3f_4::is_declaration_name;
use tsox_frontend::ast::mig::m3g::is_label_name;
use tsox_frontend::ast::mig::m3g_2::is_tag_name;
use tsox_frontend::ast::mig::m3g_3::is_part_of_type_node;
use tsox_frontend::ast::{self, Node, NodeData, SourceFile, Symbol, SyntaxKind};
use tsox_frontend::scanner;

use super::m5s::{DisplayPartsWriter, SpanFidelity, VSClassifiedTextRun};

pub const SYMBOL_FORMAT_FLAGS: u32 = 0;
pub const TYPE_FORMAT_FLAGS: TypeFormatFlags = TypeFormatFlags::NONE;

pub type DocumentationLocationMapper<'a> = &'a dyn Fn(&Arc<SourceFile>, TextRange) -> (Location, SpanFidelity);

pub struct SymbolDisplayInfo {
    pub display_parts: DisplayPartsWriter,
    pub declaration: Option<Arc<Node>>,
}

impl LanguageService {
    pub fn documentation_location_mapper(
        &self,
        feature: super::m5s::SpanFeature,
    ) -> impl Fn(&Arc<SourceFile>, TextRange) -> (Location, SpanFidelity) + '_ {
        move |file, file_range| self.source_file_range_to_lsp_location_for_feature(file, file_range, feature)
    }

    pub fn get_quick_info_and_documentation_for_symbol(
        &self,
        c: &mut Checker,
        symbol: Option<&Arc<Symbol>>,
        node: &Arc<Node>,
        content_format: &str,
        vc: Option<&mut tsox_checker::checker::mig::m2f::VerbosityContext>,
        vs_capability: bool,
    ) -> (String, String, String, Vec<VSClassifiedTextRun>) {
        let info = get_quick_info_and_declaration_at_location(
            c,
            symbol,
            node,
            vc,
            vs_capability,
            crate::ls::utilities::get_meaning_from_location(node),
        );
        let quick_info = info.display_parts.to_string();
        if quick_info.is_empty() {
            return (String::new(), String::new(), String::new(), Vec::new());
        }
        let quick_info_runs = info.display_parts.get_runs().to_vec();

        let mapper = self.documentation_location_mapper(super::m5s::SpanFeature::Definition);
        let documentation = get_documentation_for_symbol(
            &mapper,
            c,
            symbol,
            node,
            info.declaration.as_ref(),
            content_format,
            false,
        );

        let mut vs_documentation = String::new();
        if vs_capability {
            vs_documentation = get_documentation_for_symbol(
                &mapper,
                c,
                symbol,
                node,
                info.declaration.as_ref(),
                "plaintext",
                true,
            );
        }

        (quick_info, documentation, vs_documentation, quick_info_runs)
    }
}

pub fn get_quick_info_and_declaration_at_location(
    c: &mut Checker,
    symbol: Option<&Arc<Symbol>>,
    node: &Arc<Node>,
    vc: Option<&mut tsox_checker::checker::mig::m2f::VerbosityContext>,
    vs_capability: bool,
    meaning: u32,
) -> SymbolDisplayInfo {
    let container = crate::ls::utilities::get_container_node(node);
    let mut dpw = DisplayPartsWriter::new(vs_capability);
    let source_file = ast::get_source_file_of_node(node);

    if (node.kind == SyntaxKind::ThisKeyword && is_in_expression_context(node))
        || ast::is_this_in_type_query(node)
    {
        dpw.raw_write("this");
        let t = c.get_type_at_location(node);
        write_type_classified(c, &mut dpw, &t, container.as_ref(), TYPE_FORMAT_FLAGS);
        return SymbolDisplayInfo {
            display_parts: dpw,
            declaration: None,
        };
    }
    let Some(symbol) = symbol else {
        if should_get_type(node) {
            let t = c.get_type_at_location(node);
            write_type_classified(c, &mut dpw, &t, container.as_ref(), TYPE_FORMAT_FLAGS);
        }
        return SymbolDisplayInfo {
            display_parts: dpw,
            declaration: None,
        };
    };
    let variable_property_or_accessor = ast::SymbolFlags::VARIABLE
        .union(ast::SymbolFlags::Property)
        .union(ast::SymbolFlags::ACCESSOR);
    if symbol.flags.intersects(variable_property_or_accessor) {
        dpw.raw_write(&symbol.name);
        if let Some(decl) = core_or_first_declaration(symbol) {
            return SymbolDisplayInfo {
                display_parts: dpw,
                declaration: Some(decl),
            };
        }
    }
    SymbolDisplayInfo {
        display_parts: dpw,
        declaration: core_or_first_declaration(symbol),
    }
}

fn core_or_first_declaration(symbol: &Arc<Symbol>) -> Option<Arc<Node>> {
    symbol
        .value_declaration
        .clone()
        .or_else(|| symbol.declarations.first().cloned())
}

fn write_type_classified(
    c: &mut Checker,
    dpw: &mut DisplayPartsWriter,
    t: &Arc<tsox_checker::checker::Type>,
    _enclosing: Option<&Arc<Node>>,
    flags: TypeFormatFlags,
) {
    let flags = flags.union(TypeFormatFlags::MULTILINE_OBJECT_LITERALS);
    dpw.raw_write(&c.type_to_string_ex(t, flags));
}

pub fn should_get_type(node: &Arc<Node>) -> bool {
    match node.kind {
        SyntaxKind::Identifier => {
            !(node.flags.contains(ast::NodeFlags::JSDoc) && is_declaration_name(node))
                && !is_label_name(node)
                && !is_tag_name(node)
                && !node
                    .parent()
                    .map(|p| is_const_type_reference(&p))
                    .unwrap_or(false)
        }
        SyntaxKind::ThisKeyword | SyntaxKind::ThisType | SyntaxKind::SuperKeyword | SyntaxKind::NamedTupleMember => true,
        SyntaxKind::MetaProperty => is_import_meta(node),
        _ => false,
    }
}

pub fn type_parameter_to_string(
    c: &mut Checker,
    t: &Arc<tsox_checker::checker::Type>,
    enclosing_declaration: Option<&Arc<Node>>,
    vc: Option<&mut tsox_checker::checker::mig::m2f::VerbosityContext>,
) -> String {
    c.type_parameter_to_string_ex(t, enclosing_declaration, vc)
}

pub fn contains_typedef_tag(jsdoc: &Arc<Node>) -> bool {
    if jsdoc.kind == SyntaxKind::JSDoc {
        if let Some(tags) = jsdoc_tags(jsdoc) {
            for tag in &tags.nodes {
                if tag.kind == SyntaxKind::JSDocTypedefTag || tag.kind == SyntaxKind::JSDocCallbackTag {
                    return true;
                }
            }
        }
    }
    false
}

pub fn documentation_from_signature(
    get_mapped_location: DocumentationLocationMapper,
    c: &mut Checker,
    symbol: Option<&Arc<Symbol>>,
    node: Option<&Arc<Node>>,
    location: &Arc<Node>,
    content_format: &str,
    comment_only: bool,
) -> String {
    let Some(node) = node else {
        return String::new();
    };
    let Some(signature) = c.get_resolved_signature(node) else {
        return String::new();
    };
    let Some(declaration) = signature.declaration.clone() else {
        return String::new();
    };
    if ast::is_call_signature_declaration(&declaration) || ast::is_construct_signature_declaration(&declaration) {
        return get_documentation_from_declaration(
            get_mapped_location,
            c,
            symbol,
            Some(&declaration),
            location,
            content_format,
            comment_only,
        );
    }
    String::new()
}

pub fn documentation_from_alias(
    get_mapped_location: DocumentationLocationMapper,
    c: &mut Checker,
    symbol: Option<&Arc<Symbol>>,
    node: &Arc<Node>,
    content_format: &str,
    comment_only: bool,
) -> String {
    let Some(symbol) = symbol else {
        return String::new();
    };
    if !symbol.flags.intersects(ast::SymbolFlags::Alias) {
        return String::new();
    }
    let aliased_symbol = c.get_aliased_symbol(symbol);
    if c.is_unknown_symbol(&aliased_symbol) {
        return String::new();
    }
    let mut candidates: Vec<Arc<Symbol>> = vec![Arc::clone(&aliased_symbol)];
    if let Some(export_symbol) = &aliased_symbol.export_symbol {
        candidates.push(Arc::clone(export_symbol));
    }
    for candidate in &candidates {
        let aliased_declaration = candidate
            .value_declaration
            .clone()
            .or_else(|| candidate.declarations.first().cloned());
        let Some(aliased_declaration) = aliased_declaration else {
            continue;
        };
        let documentation = get_documentation_from_declaration(
            get_mapped_location,
            c,
            Some(candidate),
            Some(&aliased_declaration),
            node,
            content_format,
            comment_only,
        );
        if !documentation.is_empty() {
            return documentation;
        }
    }
    String::new()
}

pub fn documentation_from_root_symbols(
    get_mapped_location: DocumentationLocationMapper,
    c: &mut Checker,
    symbol: Option<&Arc<Symbol>>,
    node: &Arc<Node>,
    content_format: &str,
    comment_only: bool,
) -> String {
    let Some(symbol) = symbol else {
        return String::new();
    };
    let root_symbols = c.get_root_symbols(symbol);
    if root_symbols.len() <= 1 {
        return String::new();
    }
    let mut docs: Vec<String> = Vec::new();
    for root_symbol in &root_symbols {
        let mut declarations: Vec<Arc<Node>> = root_symbol.declarations.clone();
        if declarations.is_empty() {
            if let Some(value_declaration) = &root_symbol.value_declaration {
                declarations.push(Arc::clone(value_declaration));
            }
        }
        for declaration in &declarations {
            let documentation = get_documentation_from_declaration(
                get_mapped_location,
                c,
                Some(root_symbol),
                Some(declaration),
                node,
                content_format,
                comment_only,
            );
            if !documentation.is_empty() && !docs.contains(&documentation) {
                docs.push(documentation);
            }
        }
    }
    docs.join("\n")
}

pub fn get_documentation_from_declaration(
    get_mapped_location: DocumentationLocationMapper,
    c: &mut Checker,
    symbol: Option<&Arc<Symbol>>,
    declaration: Option<&Arc<Node>>,
    location: &Arc<Node>,
    content_format: &str,
    comment_only: bool,
) -> String {
    let Some(declaration) = declaration else {
        return String::new();
    };
    let is_markdown = content_format == "markdown";
    let mut b = String::new();
    if let Some(jsdoc) = get_jsdoc_or_tag(c, declaration) {
        if declaration.flags.contains(ast::NodeFlags::Reparsed) || !contains_typedef_tag(&jsdoc) {
            let jsdoc_comments = jsdoc_comment_nodes(&jsdoc);
            write_comments(get_mapped_location, &mut b, c, jsdoc_comments, is_markdown);
            if jsdoc.kind == SyntaxKind::JSDoc && !comment_only {
                let Some(tags) = jsdoc_tags(&jsdoc) else {
                    return b;
                };
                for tag in &tags.nodes {
                    if tag.kind == SyntaxKind::JSDocTypeTag
                        || tag.kind == SyntaxKind::JSDocTypedefTag
                        || tag.kind == SyntaxKind::JSDocCallbackTag
                    {
                        continue;
                    }
                    b.push_str("\n\n");
                    let tag_name_text = tag_tag_name(tag).map(|n| n.text()).unwrap_or_default();
                    if is_markdown {
                        b.push_str("*@");
                        b.push_str(tag_name_text);
                        b.push('*');
                    } else {
                        b.push('@');
                        b.push_str(tag_name_text);
                    }
                    match tag.kind {
                        SyntaxKind::JSDocParameterTag | SyntaxKind::JSDocPropertyTag => {
                            write_optional_entity_name(&mut b, tag.name())
                        }
                        SyntaxKind::JSDocAugmentsTag => write_optional_entity_name(&mut b, tag_class_name(tag)),
                        SyntaxKind::JSDocTemplateTag => {
                            for (i, tp) in tag_type_parameters(tag).iter().enumerate() {
                                if i != 0 {
                                    b.push(',');
                                }
                                write_optional_entity_name(&mut b, tp.name());
                            }
                        }
                        _ => {}
                    }
                    let comments = jsdoc_comment_nodes(tag);
                    if tag.kind == SyntaxKind::JSDocUnknownTag && tag_name_text == "example" {
                        let mut comment_text = scanner::mig::m3i::get_text_of_jsdoc_comment(
                            jsdoc_comment_list(tag).map(|l| l.as_ref()),
                        );
                        if let Some(comment_text_ref) = comment_text.strip_prefix("<caption>") {
                            if let Some(caption_end) = comment_text.find("</caption>") {
                                b.push_str(" — ");
                                b.push_str(&comment_text["<caption>".len()..caption_end]);
                                comment_text = comment_text[caption_end + "</caption>".len()..].to_string();
                                loop {
                                    let s1 = comment_text.trim_start_matches([' ', '\t']);
                                    let s2 = s1.trim_start_matches(['\r', '\n']);
                                    if s1.len() == s2.len() {
                                        break;
                                    }
                                    comment_text = s2.to_string();
                                }
                            }
                        }
                        b.push('\n');
                        if comment_text.len() > 6
                            && comment_text.starts_with("```")
                            && comment_text.ends_with("```")
                            && comment_text.contains('\n')
                        {
                            b.push_str(&comment_text);
                            b.push('\n');
                        } else {
                            write_code(&mut b, "tsx", &comment_text);
                        }
                    } else if tag.kind == SyntaxKind::JSDocSeeTag {
                        if let Some(name_expression) = tag_name_expression(tag) {
                            if let Some(name) = name_expression.name() {
                                b.push_str(" — ");
                                write_name_link(get_mapped_location, &mut b, c, name, "", false, is_markdown);
                                if !comments.is_empty() {
                                    b.push(' ');
                                    write_comments(get_mapped_location, &mut b, c, comments, is_markdown);
                                }
                            }
                        }
                    } else if tag.kind == SyntaxKind::JSDocThrowsTag {
                        if let Some(type_expression) = tsox_frontend::ast::mig::m3c::type_expression(tag) {
                            b.push_str(" — ");
                            b.push_str(&scanner::mig::m3i::get_text_of_node(type_expression));
                            if !comments.is_empty() {
                                b.push(' ');
                                write_comments(get_mapped_location, &mut b, c, comments, is_markdown);
                            }
                        }
                    } else if !comments.is_empty() {
                        b.push(' ');
                        if comments[0].kind != SyntaxKind::JSDocText || !comments[0].text().starts_with('-') {
                            b.push_str("— ");
                        }
                        write_comments(get_mapped_location, &mut b, c, comments, is_markdown);
                    }
                }
            }
        }
    }
    b
}

pub fn write_code(b: &mut String, lang: &str, code: &str) {
    if code.is_empty() {
        return;
    }
    let mut ticks = 3;
    let mut fence = "`".repeat(ticks);
    while code.contains(&fence) {
        ticks += 1;
        fence = "`".repeat(ticks);
    }
    for _ in 0..ticks {
        b.push('`');
    }
    b.push_str(lang);
    b.push('\n');
    b.push_str(code);
    b.push('\n');
    for _ in 0..ticks {
        b.push('`');
    }
    b.push('\n');
}

pub fn write_comments(
    get_mapped_location: DocumentationLocationMapper,
    b: &mut String,
    c: &mut Checker,
    comments: &[Arc<Node>],
    is_markdown: bool,
) {
    for comment in comments {
        match comment.kind {
            SyntaxKind::JSDocText => b.push_str(&comment.text()),
            SyntaxKind::JSDocLink | SyntaxKind::JSDocLinkPlain => {
                write_jsdoc_link(get_mapped_location, b, c, comment, false, is_markdown)
            }
            SyntaxKind::JSDocLinkCode => write_jsdoc_link(get_mapped_location, b, c, comment, true, is_markdown),
            _ => {}
        }
    }
}

pub fn write_jsdoc_link(
    get_mapped_location: DocumentationLocationMapper,
    b: &mut String,
    c: &mut Checker,
    link: &Arc<Node>,
    quote: bool,
    is_markdown: bool,
) {
    let name = link.name();
    let text = link.text().trim().to_string();
    let Some(name) = name else {
        write_quoted_string(b, &text, quote && is_markdown);
        return;
    };
    if ast::is_identifier(&name) && (name.text() == "http" || name.text() == "https") && text.starts_with("://") {
        let mut link_text = format!("{}{}", name.text(), text);
        let mut link_uri = link_text.clone();
        if let Some(comment_pos) = link_text.find([' ', '|']) {
            link_uri = link_text[..comment_pos].to_string();
            link_text = trim_comment_prefix(&link_text[comment_pos..]).to_string();
            if link_text.is_empty() {
                link_text = link_uri.clone();
            }
        }
        if is_markdown {
            write_markdown_link(b, &link_text, &link_uri, quote);
        } else {
            write_quoted_string(b, &link_text, false);
            if link_text != link_uri {
                b.push_str(" (");
                b.push_str(&link_uri);
                b.push(')');
            }
        }
        return;
    }
    write_name_link(get_mapped_location, b, c, &name, &text, quote, is_markdown);
}

pub fn write_name_link(
    get_mapped_location: DocumentationLocationMapper,
    b: &mut String,
    c: &mut Checker,
    name: &Arc<Node>,
    text: &str,
    quote: bool,
    is_markdown: bool,
) {
    let declarations = super::m5s::get_declarations_from_location(c, name);
    if let Some(declaration) = declarations.first()
        && let Some(file) = c.get_source_file_of_node(declaration)
    {
        let node = ast::get_name_of_declaration(declaration).unwrap_or_else(|| declaration.clone());
        let (loc, fidelity) = get_mapped_location(&file, crate::ls::utilities::create_range_from_node(&node, &file));
        let prefix_len = if text.starts_with("()") { 2 } else { 0 };
        let mut link_text = trim_comment_prefix(&text[prefix_len..]).to_string();
        if link_text.is_empty() {
            link_text = format!("{}{}", get_entity_name_string(name), &text[..prefix_len]);
        }
        if is_markdown && fidelity == SpanFidelity::SingleSegment {
            let link_uri = format!(
                "{}#{},{}-{},{}",
                loc.uri.0,
                loc.range.start.line + 1,
                loc.range.start.character + 1,
                loc.range.end.line + 1,
                loc.range.end.character + 1
            );
            write_markdown_link(b, &link_text, &link_uri, quote);
        } else {
            write_quoted_string(b, &link_text, false);
        }
        return;
    }
    let suffix = if !text.is_empty() { format!(" {}", text) } else { String::new() };
    write_quoted_string(b, &format!("{}{}", get_entity_name_string(name), suffix), quote && is_markdown);
}

pub fn trim_comment_prefix(text: &str) -> &str {
    text.trim_start_matches(' ')
        .strip_prefix('|')
        .unwrap_or(text.trim_start_matches(' '))
        .trim_start_matches(' ')
        .trim_start_matches(' ')
}

pub fn write_markdown_link(b: &mut String, text: &str, uri: &str, quote: bool) {
    b.push('[');
    write_quoted_string(b, text, quote);
    b.push_str("](");
    b.push_str(uri);
    b.push(')');
}

pub fn write_optional_entity_name(b: &mut String, name: Option<&Arc<Node>>) {
    if let Some(name) = name {
        b.push(' ');
        write_quoted_string(b, &get_entity_name_string(name), true);
    }
}

pub fn write_quoted_string(b: &mut String, s: &str, quote: bool) {
    if quote && !s.contains('`') {
        b.push('`');
        b.push_str(s);
        b.push('`');
    } else {
        b.push_str(s);
    }
}

pub fn get_entity_name_string(name: &Arc<Node>) -> String {
    let mut b = String::new();
    write_entity_name_parts(&mut b, name);
    b
}

pub fn write_entity_name_parts(b: &mut String, node: &Arc<Node>) {
    match node.kind {
        SyntaxKind::Identifier => b.push_str(&node.text()),
        SyntaxKind::QualifiedName => {
            let NodeData::QualifiedName(d) = &node.data else {
                return;
            };
            write_entity_name_parts(b, &d.left);
            b.push('.');
            write_entity_name_parts(b, &d.right);
        }
        SyntaxKind::PropertyAccessExpression => {
            if let Some(expression) = node.expression() {
                write_entity_name_parts(b, expression);
            }
            b.push('.');
            if let Some(name) = node.name() {
                write_entity_name_parts(b, name);
            }
        }
        SyntaxKind::ParenthesizedExpression | SyntaxKind::ExpressionWithTypeArguments => {
            if let Some(expression) = node.expression() {
                write_entity_name_parts(b, expression);
            }
        }
        SyntaxKind::JSDocNameReference => {
            if let Some(name) = node.name() {
                write_entity_name_parts(b, name);
            }
        }
        _ => {}
    }
}

pub fn get_documentation_for_symbol(
    get_mapped_location: DocumentationLocationMapper,
    c: &mut Checker,
    symbol: Option<&Arc<Symbol>>,
    node: &Arc<Node>,
    declaration: Option<&Arc<Node>>,
    content_format: &str,
    comment_only: bool,
) -> String {
    let documentation = documentation_from_signature(
        get_mapped_location,
        c,
        symbol,
        get_call_or_new_expression(node).as_ref(),
        node,
        content_format,
        comment_only,
    );
    if !documentation.is_empty() {
        return documentation;
    }
    let documentation =
        documentation_from_root_symbols(get_mapped_location, c, symbol, node, content_format, comment_only);
    if !documentation.is_empty() {
        return documentation;
    }
    let documentation =
        get_documentation_from_declaration(get_mapped_location, c, symbol, declaration, node, content_format, comment_only);
    if !documentation.is_empty() {
        return documentation;
    }
    documentation_from_alias(get_mapped_location, c, symbol, node, content_format, comment_only)
}

fn get_call_or_new_expression(node: &Arc<Node>) -> Option<Arc<Node>> {
    if node.kind == SyntaxKind::SourceFile {
        return None;
    }
    let mut node = Arc::clone(node);
    if let Some(parent) = node.parent() {
        if ast::is_property_access_expression(&parent) && parent.name().is_some_and(|n| Arc::ptr_eq(n, &node)) {
            node = parent;
        }
    }
    if let Some(parent) = node.parent() {
        if (ast::is_call_expression(&parent) || ast::is_new_expression(&parent))
            && parent.expression().is_some_and(|e| Arc::ptr_eq(e, &node))
        {
            return Some(parent);
        }
    }
    None
}

pub fn get_jsdoc_or_tag(c: &mut Checker, node: &Arc<Node>) -> Option<Arc<Node>> {
    get_jsdoc_or_tag_with_seen(c, node, &mut std::collections::HashSet::new())
}

fn get_jsdoc_or_tag_with_seen(
    c: &mut Checker,
    node: &Arc<Node>,
    seen: &mut std::collections::HashSet<usize>,
) -> Option<Arc<Node>> {
    if let Some(jsdoc) = get_jsdoc(c, node) {
        return Some(jsdoc);
    }
    let parent = node.parent()?;
    if ast::is_parameter_declaration(node) {
        let name = node.name();
        if name.is_some_and(|n| ast::is_binding_pattern(n)) {
            return get_jsdoc_parameter_tag_by_position(c, node, seen);
        }
        let text = name.map(|n| n.text()).unwrap_or_default();
        return get_matching_jsdoc_tag(c, &parent, &text, is_matching_parameter_tag, seen);
    }
    if ast::is_type_parameter_declaration(node) {
        let text = node.name().map(|n| n.text()).unwrap_or_default();
        return get_matching_jsdoc_tag(c, &parent, &text, is_matching_template_tag, seen);
    }
    if ast::is_variable_declaration(node)
        && ast::is_variable_declaration_list(&parent)
        && match &parent.data {
            NodeData::VariableDeclarationList(d) => d
                .declarations
                .nodes
                .first()
                .is_some_and(|first| Arc::ptr_eq(first, node)),
            _ => false,
        }
    {
        let grandparent = parent.parent()?;
        return get_jsdoc_or_tag_with_seen(c, &grandparent, seen);
    }
    let is_fn_or_class_expr = node.kind == SyntaxKind::FunctionExpression
        || node.kind == SyntaxKind::ArrowFunction
        || ast::is_class_expression(node);
    if is_fn_or_class_expr
        && (ast::is_variable_declaration(&parent)
            || ast::is_property_declaration(&parent)
            || ast::is_property_assignment(&parent))
        && parent.initializer().is_some_and(|i| Arc::ptr_eq(i, node))
    {
        return get_jsdoc_or_tag_with_seen(c, &parent, seen);
    }
    if ast::is_binding_element(node) && ast::is_object_binding_pattern(&parent) {
        if let Some(name) = node.name() {
            if ast::is_identifier(name) {
                let object_type = c.get_type_at_location(&parent);
                if let Some(prop) = c.get_property_of_type(&object_type, &name.text()) {
                    for d in &prop.declarations {
                        if let Some(jsdoc) = get_jsdoc(c, d) {
                            return Some(jsdoc);
                        }
                    }
                }
            }
        }
    }
    if let Some(symbol) = tsox_checker::checker::mig::m1a::r19k2_defs::symbol_of_node(node) {
        if ast::is_function_declaration(node)
            || ast::is_method_declaration(node)
            || ast::is_method_signature_declaration(node)
            || ast::is_constructor_declaration(node)
            || ast::is_construct_signature_declaration(node)
        {
            if let Some(first_signature) = symbol.declarations.iter().find(|d| ast::is_function_like(d)) {
                if !Arc::ptr_eq(first_signature, node) {
                    if let Some(jsdoc) = get_jsdoc_or_tag_with_seen(c, first_signature, seen) {
                        return Some(jsdoc);
                    }
                }
            }
        }
    }
    let _ = seen;
    None
}

fn get_matching_jsdoc_tag(
    c: &mut Checker,
    node: &Arc<Node>,
    name: &str,
    match_fn: fn(&Arc<Node>, &str) -> bool,
    seen: &mut std::collections::HashSet<usize>,
) -> Option<Arc<Node>> {
    let jsdoc = get_jsdoc_or_tag_with_seen(c, node, seen)?;
    if jsdoc.kind != SyntaxKind::JSDoc {
        return None;
    }
    let tags = jsdoc_tags(&jsdoc)?;
    for tag in &tags.nodes {
        if match_fn(tag, name) {
            return Some(Arc::clone(tag));
        }
    }
    None
}

fn get_jsdoc_parameter_tag_by_position(
    c: &mut Checker,
    param: &Arc<Node>,
    seen: &mut std::collections::HashSet<usize>,
) -> Option<Arc<Node>> {
    let parent = param.parent()?;
    let param_index = parent.parameters()?.nodes.iter().position(|p| Arc::ptr_eq(p, param))?;
    let jsdoc = get_jsdoc_or_tag_with_seen(c, &parent, seen)?;
    if jsdoc.kind != SyntaxKind::JSDoc {
        return None;
    }
    let tags = jsdoc_tags(&jsdoc)?;
    let mut param_tag_index = 0usize;
    for tag in &tags.nodes {
        if tag.kind == SyntaxKind::JSDocParameterTag {
            if param_tag_index == param_index {
                return Some(Arc::clone(tag));
            }
            param_tag_index += 1;
        }
    }
    None
}

fn is_matching_parameter_tag(tag: &Arc<Node>, name: &str) -> bool {
    tag.kind == SyntaxKind::JSDocParameterTag && is_node_with_name(tag, name)
}

fn is_matching_template_tag(tag: &Arc<Node>, name: &str) -> bool {
    tag.kind == SyntaxKind::JSDocTemplateTag
        && tag_type_parameters(tag).iter().any(|tp| is_node_with_name(tp, name))
}

fn is_node_with_name(node: &Arc<Node>, name: &str) -> bool {
    node.name()
        .is_some_and(|n| ast::is_identifier(n) && n.text() == name)
}

fn get_jsdoc(c: &Checker, node: &Arc<Node>) -> Option<Arc<Node>> {
    let file = c.get_source_file_of_node(node)?;
    node.jsdoc(&file).last().cloned()
}

fn jsdoc_tags(jsdoc: &Arc<Node>) -> Option<&Arc<tsox_frontend::ast::NodeList>> {
    match &jsdoc.data {
        NodeData::JSDoc(d) => d.tags.as_ref(),
        _ => None,
    }
}

fn jsdoc_comment_list(node: &Arc<Node>) -> Option<&Arc<tsox_frontend::ast::NodeList>> {
    match &node.data {
        NodeData::JSDoc(d) => Some(&d.comment),
        NodeData::JSDocUnknownTag(d) => d.comment.as_ref(),
        NodeData::JSDocAugmentsTag(d) => d.comment.as_ref(),
        NodeData::JSDocImplementsTag(d) => d.comment.as_ref(),
        NodeData::JSDocDeprecatedTag(d) => d.comment.as_ref(),
        NodeData::JSDocPublicTag(d) => d.comment.as_ref(),
        NodeData::JSDocPrivateTag(d) => d.comment.as_ref(),
        NodeData::JSDocProtectedTag(d) => d.comment.as_ref(),
        NodeData::JSDocReadonlyTag(d) => d.comment.as_ref(),
        NodeData::JSDocOverrideTag(d) => d.comment.as_ref(),
        NodeData::JSDocSeeTag(d) => d.comment.as_ref(),
        NodeData::JSDocSatisfiesTag(d) => d.comment.as_ref(),
        NodeData::JSDocThrowsTag(d) => d.comment.as_ref(),
        NodeData::JSDocTypeTag(d) => d.comment.as_ref(),
        NodeData::JSDocReturnTag(d) => d.comment.as_ref(),
        NodeData::JSDocThisTag(d) => d.comment.as_ref(),
        NodeData::JSDocTemplateTag(d) => d.comment.as_ref(),
        NodeData::JSDocTypedefTag(d) => d.comment.as_ref(),
        NodeData::JSDocCallbackTag(d) => d.comment.as_ref(),
        NodeData::JSDocOverloadTag(d) => d.comment.as_ref(),
        NodeData::JSDocImportTag(d) => d.comment.as_ref(),
        NodeData::JSDocParameterOrPropertyTag(d) => d.comment.as_ref(),
        _ => None,
    }
}

fn jsdoc_comment_nodes(node: &Arc<Node>) -> &[Arc<Node>] {
    jsdoc_comment_list(node)
        .map(|l| l.nodes.as_slice())
        .unwrap_or(&[])
}

fn tag_tag_name(tag: &Arc<Node>) -> Option<&Arc<Node>> {
    match &tag.data {
        NodeData::JSDocUnknownTag(d) => Some(&d.tag_name),
        NodeData::JSDocAugmentsTag(d) => Some(&d.tag_name),
        NodeData::JSDocImplementsTag(d) => Some(&d.tag_name),
        NodeData::JSDocDeprecatedTag(d) => Some(&d.tag_name),
        NodeData::JSDocPublicTag(d) => Some(&d.tag_name),
        NodeData::JSDocPrivateTag(d) => Some(&d.tag_name),
        NodeData::JSDocProtectedTag(d) => Some(&d.tag_name),
        NodeData::JSDocReadonlyTag(d) => Some(&d.tag_name),
        NodeData::JSDocOverrideTag(d) => Some(&d.tag_name),
        NodeData::JSDocSeeTag(d) => Some(&d.tag_name),
        NodeData::JSDocSatisfiesTag(d) => Some(&d.tag_name),
        NodeData::JSDocThrowsTag(d) => Some(&d.tag_name),
        NodeData::JSDocTypeTag(d) => Some(&d.tag_name),
        NodeData::JSDocReturnTag(d) => Some(&d.tag_name),
        NodeData::JSDocThisTag(d) => Some(&d.tag_name),
        NodeData::JSDocTemplateTag(d) => Some(&d.tag_name),
        NodeData::JSDocTypedefTag(d) => Some(&d.tag_name),
        NodeData::JSDocCallbackTag(d) => Some(&d.tag_name),
        NodeData::JSDocOverloadTag(d) => Some(&d.tag_name),
        NodeData::JSDocImportTag(d) => Some(&d.tag_name),
        NodeData::JSDocParameterOrPropertyTag(d) => Some(&d.tag_name),
        _ => None,
    }
}

fn tag_class_name(tag: &Arc<Node>) -> Option<&Arc<Node>> {
    match &tag.data {
        NodeData::JSDocAugmentsTag(d) => Some(&d.class_name),
        NodeData::JSDocImplementsTag(d) => Some(&d.class_name),
        _ => None,
    }
}

fn tag_name_expression(tag: &Arc<Node>) -> Option<&Arc<Node>> {
    match &tag.data {
        NodeData::JSDocSeeTag(d) => Some(&d.name_expression),
        _ => None,
    }
}

fn tag_type_parameters(tag: &Arc<Node>) -> &Arc<tsox_frontend::ast::NodeList> {
    match &tag.data {
        NodeData::JSDocTemplateTag(d) => &d.type_parameters,
        _ => panic!("tag_type_parameters on non-template tag"),
    }
}

pub fn is_const_type_reference(node: &Arc<Node>) -> bool {
    match &node.data {
        tsox_frontend::ast::NodeData::TypeReferenceNode(d) => {
            d.type_arguments.is_none() && ast::is_identifier(&d.type_name) && d.type_name.text() == "const"
        }
        _ => false,
    }
}

pub fn is_in_expression_context(node: &Arc<Node>) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    match parent.kind {
        SyntaxKind::VariableDeclaration
        | SyntaxKind::Parameter
        | SyntaxKind::PropertyDeclaration
        | SyntaxKind::PropertySignature
        | SyntaxKind::EnumMember
        | SyntaxKind::PropertyAssignment
        | SyntaxKind::BindingElement => {
            parent
                .initializer()
                .is_some_and(|i| Arc::ptr_eq(i, node))
        }
        SyntaxKind::ExpressionStatement
        | SyntaxKind::IfStatement
        | SyntaxKind::DoStatement
        | SyntaxKind::WhileStatement
        | SyntaxKind::ReturnStatement
        | SyntaxKind::WithStatement
        | SyntaxKind::SwitchStatement
        | SyntaxKind::CaseClause
        | SyntaxKind::DefaultClause
        | SyntaxKind::ThrowStatement
        | SyntaxKind::TypeAssertionExpression
        | SyntaxKind::AsExpression
        | SyntaxKind::TemplateSpan
        | SyntaxKind::ComputedPropertyName
        | SyntaxKind::SatisfiesExpression => {
            parent
                .expression()
                .is_some_and(|e| Arc::ptr_eq(e, node))
        }
        SyntaxKind::ForStatement => match &parent.data {
            NodeData::ForStatement(d) => {
                d.initializer.as_ref().is_some_and(|i| {
                    Arc::ptr_eq(i, node) && i.kind != SyntaxKind::VariableDeclarationList
                }) || d
                    .condition
                    .as_ref()
                    .is_some_and(|c| Arc::ptr_eq(c, node))
                    || d.incrementor.as_ref().is_some_and(|i| Arc::ptr_eq(i, node))
            }
            _ => false,
        },
        SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement => match &parent.data {
            NodeData::ForInOrOfStatement(d) => {
                (Arc::ptr_eq(&d.initializer, node)
                    && d.initializer.kind != SyntaxKind::VariableDeclarationList)
                    || Arc::ptr_eq(&d.expression, node)
            }
            _ => false,
        },
        SyntaxKind::Decorator
        | SyntaxKind::JsxExpression
        | SyntaxKind::JsxSpreadAttribute
        | SyntaxKind::SpreadAssignment => true,
        SyntaxKind::ExpressionWithTypeArguments => {
            parent
                .expression()
                .is_some_and(|e| Arc::ptr_eq(e, node))
                && !is_part_of_type_node(&parent)
        }
        SyntaxKind::ShorthandPropertyAssignment => match &parent.data {
            NodeData::ShorthandPropertyAssignment(d) => d
                .object_assignment_initializer
                .as_ref()
                .is_some_and(|i| Arc::ptr_eq(i, node)),
            _ => false,
        },
        _ => tsox_checker::checker::mig::wc3_6::r24k4_defs::is_expression_node_r24k4(&parent),
    }
}
