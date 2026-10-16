#![allow(dead_code, unused_imports, unused_variables)]

use std::cmp::Ordering;
use std::sync::Arc;

use tsox_checker::checker::Checker;
use tsox_checker::checker::types::SignatureKind;
use tsox_checker::checker::types::Type;
use tsox_checker::checker::types::TypeFlags;
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::{self, Node, SourceFile, SyntaxKind};
use tsox_frontend::scanner;

use crate::ls::language_service::LanguageService;
use crate::lsp::lsproto_lsp::Range;

type ClientCapabilities = crate::mig::m5m::ResolvedClientCapabilities;

fn semantic_tokens_client_token_types(caps: &ClientCapabilities) -> Vec<String> { ::tsox_core::fntrace::enter("semantic_tokens_client_token_types"); 
    caps.raw
        .pointer("/textDocument/semanticTokens/tokenTypes")
        .and_then(|v| v.as_array())
        .map(|array| {
            array
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn semantic_tokens_client_token_modifiers(caps: &ClientCapabilities) -> Vec<String> { ::tsox_core::fntrace::enter("semantic_tokens_client_token_modifiers"); 
    caps.raw
        .pointer("/textDocument/semanticTokens/tokenModifiers")
        .and_then(|v| v.as_array())
        .map(|array| {
            array
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

pub type SpanMapFeature = u32;
pub const SPANMAP_FEATURE_SEMANTIC_TOKENS: SpanMapFeature = 2;
pub const SPANMAP_FIDELITY_EXACT: u32 = 1;

pub static TOKEN_TYPE_NAMES: &[&str] = &[
    "namespace",
    "class",
    "enum",
    "interface",
    "struct",
    "typeParameter",
    "type",
    "parameter",
    "variable",
    "property",
    "enumMember",
    "decorator",
    "event",
    "function",
    "method",
    "macro",
    "label",
    "comment",
    "string",
    "keyword",
    "number",
    "regexp",
    "operator",
];

pub static TOKEN_MODIFIER_NAMES: &[&str] = &[
    "declaration",
    "definition",
    "readonly",
    "static",
    "deprecated",
    "abstract",
    "async",
    "modification",
    "documentation",
    "defaultLibrary",
    "local",
];

pub const TOKEN_TYPE_NAMESPACE: u32 = 0;
pub const TOKEN_TYPE_CLASS: u32 = 1;
pub const TOKEN_TYPE_ENUM: u32 = 2;
pub const TOKEN_TYPE_INTERFACE: u32 = 3;
pub const TOKEN_TYPE_STRUCT: u32 = 4;
pub const TOKEN_TYPE_TYPE_PARAMETER: u32 = 5;
pub const TOKEN_TYPE_TYPE: u32 = 6;
pub const TOKEN_TYPE_PARAMETER: u32 = 7;
pub const TOKEN_TYPE_VARIABLE: u32 = 8;
pub const TOKEN_TYPE_PROPERTY: u32 = 9;
pub const TOKEN_TYPE_ENUM_MEMBER: u32 = 10;
pub const TOKEN_TYPE_DECORATOR: u32 = 11;
pub const TOKEN_TYPE_EVENT: u32 = 12;
pub const TOKEN_TYPE_FUNCTION: u32 = 13;
pub const TOKEN_TYPE_METHOD: u32 = 14;
pub const TOKEN_TYPE_MACRO: u32 = 15;
pub const TOKEN_TYPE_LABEL: u32 = 16;
pub const TOKEN_TYPE_COMMENT: u32 = 17;
pub const TOKEN_TYPE_STRING: u32 = 18;
pub const TOKEN_TYPE_KEYWORD: u32 = 19;
pub const TOKEN_TYPE_NUMBER: u32 = 20;
pub const TOKEN_TYPE_REGEXP: u32 = 21;
pub const TOKEN_TYPE_OPERATOR: u32 = 22;

pub const TOKEN_MODIFIER_DECLARATION: u32 = 1 << 0;
pub const TOKEN_MODIFIER_DEFINITION: u32 = 1 << 1;
pub const TOKEN_MODIFIER_READONLY: u32 = 1 << 2;
pub const TOKEN_MODIFIER_STATIC: u32 = 1 << 3;
pub const TOKEN_MODIFIER_DEPRECATED: u32 = 1 << 4;
pub const TOKEN_MODIFIER_ABSTRACT: u32 = 1 << 5;
pub const TOKEN_MODIFIER_ASYNC: u32 = 1 << 6;
pub const TOKEN_MODIFIER_MODIFICATION: u32 = 1 << 7;
pub const TOKEN_MODIFIER_DOCUMENTATION: u32 = 1 << 8;
pub const TOKEN_MODIFIER_DEFAULT_LIBRARY: u32 = 1 << 9;
pub const TOKEN_MODIFIER_LOCAL: u32 = 1 << 10;

pub struct ResolvedSemanticTokensClientCapabilities {
    pub token_types: Vec<String>,
    pub token_modifiers: Vec<String>,
}

pub struct SemanticTokensLegend {
    pub token_types: Vec<String>,
    pub token_modifiers: Vec<String>,
}

pub fn semantic_tokens_legend(
    client_capabilities: &ResolvedSemanticTokensClientCapabilities,
) -> SemanticTokensLegend { ::tsox_core::fntrace::enter("semantic_tokens_legend"); 
    let types: Vec<String> = TOKEN_TYPE_NAMES
        .iter()
        .filter(|t| client_capabilities.token_types.iter().any(|c| c == *t))
        .map(|t| t.to_string())
        .collect();
    let modifiers: Vec<String> = TOKEN_MODIFIER_NAMES
        .iter()
        .filter(|m| client_capabilities.token_modifiers.iter().any(|c| c == *m))
        .map(|m| m.to_string())
        .collect();
    SemanticTokensLegend {
        token_types: types,
        token_modifiers: modifiers,
    }
}

pub struct M5vSemanticToken {
    pub node: Arc<Node>,
    pub file: Arc<SourceFile>,
    pub token_type: u32,
    pub token_modifier: u32,
}

impl LanguageService {
    pub fn m5v_collect_semantic_tokens(
        &self,
        ctx: &crate::mig::m5m::ResolvedClientCapabilitiesContext,
        c: &mut Checker,
        file: &Arc<SourceFile>,
        program: &tsox_compile::compiler::Program,
    ) -> Vec<M5vSemanticToken> { ::tsox_core::fntrace::enter("m5v_collect_semantic_tokens"); 
        self.m5v_collect_semantic_tokens_in_range(
            ctx,
            c,
            file,
            program,
            file.node.pos(),
            file.node.end(),
        )
    }

    fn m5v_collect_semantic_tokens_in_range(
        &self,
        ctx: &crate::mig::m5m::ResolvedClientCapabilitiesContext,
        c: &mut Checker,
        file: &Arc<SourceFile>,
        program: &tsox_compile::compiler::Program,
        span_start: usize,
        span_end: usize,
    ) -> Vec<M5vSemanticToken> { ::tsox_core::fntrace::enter("m5v_collect_semantic_tokens_in_range"); 
        let raw = self.collect_semantic_tokens_in_range(c, file, program, span_start, span_end);
        raw.into_iter()
            .map(|t| M5vSemanticToken {
                node: t.node,
                file: Arc::clone(file),
                token_type: t.token_type,
                token_modifier: t.token_modifier,
            })
            .collect()
    }
}

pub fn sort_semantic_tokens(tokens: &mut [M5vSemanticToken], converters: &crate::mig::m5u_conv::M5uConverters) { ::tsox_core::fntrace::enter("sort_semantic_tokens"); 
    tokens.sort_by(|a, b| {
        let (a_range, _) = semantic_token_lsp_range(a, converters);
        let (b_range, _) = semantic_token_lsp_range(b, converters);
        let ord = a_range.start.line.cmp(&b_range.start.line);
        if ord != Ordering::Equal {
            return ord;
        }
        let ord = a_range.start.character.cmp(&b_range.start.character);
        if ord != Ordering::Equal {
            return ord;
        }
        let ord = a.file.file_name.cmp(&b.file.file_name);
        if ord != Ordering::Equal {
            return ord;
        }
        a.node.pos().cmp(&b.node.pos())
    });
}

pub fn semantic_token_lsp_range(
    token: &M5vSemanticToken,
    converters: &crate::mig::m5u_conv::M5uConverters,
) -> (Range, u32) { ::tsox_core::fntrace::enter("semantic_token_lsp_range"); 
    let start = tsox_frontend::scanner::mig::x5a::get_token_pos_of_node(
        &token.node,
        &token.file,
        false,
    );
    let script = crate::mig::m5u_conv::SourceFileScriptView {
        file: Arc::clone(&token.file),
    };
    converters.to_lsp_range_for_feature(
        &script,
        TextRange::new(start, token.node.end()),
        SPANMAP_FEATURE_SEMANTIC_TOKENS,
    )
}

pub fn reclassify_by_type(c: &mut Checker, node: &Arc<Node>, tt: u32) -> u32 { ::tsox_core::fntrace::enter("reclassify_by_type"); 
    if tt == TOKEN_TYPE_VARIABLE || tt == TOKEN_TYPE_PROPERTY || tt == TOKEN_TYPE_PARAMETER {
        let typ = c.get_type_at_location(node);
        let test = |condition: &dyn Fn(&Arc<Type>) -> bool| -> bool {
            if condition(&typ) {
                return true;
            }
            if typ.flags.contains(TypeFlags::Union) {
                if let Some(union_data) = typ.as_union_type() {
                    if union_data
                        .union_or_intersection
                        .types
                        .iter()
                        .any(|t| condition(t))
                    {
                        return true;
                    }
                }
            }
            false
        };

        if tt != TOKEN_TYPE_PARAMETER
            && test(&|t: &Arc<Type>| {
                !c.get_signatures_of_type(t, SignatureKind::Construct).is_empty()
            })
        {
            return TOKEN_TYPE_CLASS;
        }

        let has_call_signatures = test(&|t: &Arc<Type>| {
            !c.get_signatures_of_type(t, SignatureKind::Call).is_empty()
        });
        if has_call_signatures {
            let has_no_properties = !test(&|t: &Arc<Type>| {
                t.as_object_type()
                    .map_or(false, |obj| !obj.structured.properties.is_empty())
            });
            if has_no_properties || is_expression_in_call_expression(node) {
                if tt == TOKEN_TYPE_PROPERTY {
                    return TOKEN_TYPE_METHOD;
                }
                return TOKEN_TYPE_FUNCTION;
            }
        }
    }
    tt
}

pub fn is_local_declaration(decl: &Arc<Node>, source_file: &Arc<SourceFile>) -> bool { ::tsox_core::fntrace::enter("is_local_declaration"); 
    let mut decl = Arc::clone(decl);
    if ast::is_binding_element(&decl) {
        if let Some(d) = get_declaration_for_binding_element(&decl) {
            decl = d;
        }
    }
    if ast::is_variable_declaration(&decl) {
        let parent = decl.parent();
        if let Some(parent) = parent {
            if ast::is_catch_clause(&parent) {
                return is_in_same_file_as_source_file(&decl, source_file);
            }
            if ast::is_variable_declaration_list(&parent) {
                if let Some(grandparent) = parent.parent() {
                    let great_grandparent = grandparent.parent();
                    return (!great_grandparent
                        .as_ref()
                        .map_or(false, |g| ast::is_source_file(g))
                        || grandparent
                            .parent()
                            .as_ref()
                            .map_or(false, |g| ast::is_catch_clause(g)))
                        && is_in_same_file_as_source_file(&decl, source_file);
                }
            }
        }
    } else if ast::is_function_declaration(&decl) {
        let parent = decl.parent();
        return parent.is_some()
            && !parent.as_ref().map_or(true, |p| ast::is_source_file(p))
            && is_in_same_file_as_source_file(&decl, source_file);
    }
    false
}

fn is_in_same_file_as_source_file(decl: &Arc<Node>, source_file: &Arc<SourceFile>) -> bool { ::tsox_core::fntrace::enter("is_in_same_file_as_source_file"); 
    ast::get_source_file_of_node(decl).is_some_and(|n| n.id() == source_file.node.id())
}

pub fn get_declaration_for_binding_element(element: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_declaration_for_binding_element"); 
    let mut element = Arc::clone(element);
    loop {
        let parent = element.parent()?;
        if ast::is_binding_pattern(&parent) {
            if let Some(grandparent) = parent.parent() {
                if ast::is_binding_element(&grandparent) {
                    element = grandparent;
                    continue;
                }
            }
            return parent.parent();
        }
        return Some(element);
    }
}

pub fn is_in_import_clause(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_in_import_clause"); 
    let parent = node.parent();
    parent.is_some()
        && parent
            .as_ref()
            .map_or(false, |p| {
                ast::is_import_clause(p) || ast::is_import_specifier(p) || ast::is_namespace_import(p)
            })
}

pub fn is_expression_in_call_expression(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_expression_in_call_expression"); 
    let mut node = Arc::clone(node);
    while ast::mig::m3g_2::is_right_side_of_qualified_name_or_property_access(&node) {
        node = node.parent().unwrap();
    }
    let parent = node.parent();
    parent.is_some()
        && parent.as_ref().map_or(false, |p| {
            ast::is_call_expression(p) && p.expression().map_or(false, |e| Arc::ptr_eq(e, &node))
        })
}

pub fn is_infinity_or_nan_string(text: &str) -> bool { ::tsox_core::fntrace::enter("is_infinity_or_nan_string"); 
    text == "Infinity" || text == "NaN"
}

pub fn encode_semantic_tokens(
    ctx: &crate::mig::m5m::ResolvedClientCapabilitiesContext,
    tokens: &[M5vSemanticToken],
    converters: &crate::mig::m5u_conv::M5uConverters,
) -> Vec<u32> { ::tsox_core::fntrace::enter("encode_semantic_tokens"); 
    let mut type_mapping: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
    let mut modifier_mapping: std::collections::HashMap<usize, u32> = std::collections::HashMap::new();

    let client_capabilities = crate::mig::m5m::get_client_capabilities(ctx);
    let client_token_types = semantic_tokens_client_token_types(&client_capabilities);
    let client_token_modifiers = semantic_tokens_client_token_modifiers(&client_capabilities);

    let mut client_idx: u32 = 0;
    for (i, server_type) in TOKEN_TYPE_NAMES.iter().enumerate() {
        if client_token_types.iter().any(|c| c == server_type) {
            type_mapping.insert(i as u32, client_idx);
            client_idx += 1;
        }
    }

    let mut client_bit: u32 = 0;
    for (i, server_modifier) in TOKEN_MODIFIER_NAMES.iter().enumerate() {
        if client_token_modifiers
            .iter()
            .any(|c| c == server_modifier)
        {
            modifier_mapping.insert(i, client_bit);
            client_bit += 1;
        }
    }

    let mut encoded: Vec<u32> = Vec::with_capacity(tokens.len() * 5);
    let mut prev_line: u32 = 0;
    let mut prev_char: u32 = 0;

    for token in tokens {
        let Some(&client_type_idx) = type_mapping.get(&token.token_type) else {
            continue;
        };

        let mut client_modifier_mask: u32 = 0;
        for i in 0..TOKEN_MODIFIER_NAMES.len() {
            if token.token_modifier & (1 << i) != 0 {
                if let Some(&bit) = modifier_mapping.get(&i) {
                    client_modifier_mask |= 1 << bit;
                }
            }
        }

        let (lsp_range, fidelity) = semantic_token_lsp_range(token, converters);
        if fidelity != SPANMAP_FIDELITY_EXACT {
            continue;
        }

        let line = lsp_range.start.line;
        let character = lsp_range.start.character;
        let delta_line = line - prev_line;
        let delta_char = if delta_line == 0 {
            character - prev_char
        } else {
            character
        };
        let length = lsp_range.end.character - lsp_range.start.character;
        encoded.push(delta_line);
        encoded.push(delta_char);
        encoded.push(length);
        encoded.push(client_type_idx);
        encoded.push(client_modifier_mask);
        prev_line = line;
        prev_char = character;
    }

    encoded
}
