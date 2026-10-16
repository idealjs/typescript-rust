#![allow(unused_imports, dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::node_node::Node;
use tsox_frontend::ast::node_node_list::ModifierList;
use tsox_frontend::ast::node_source_file::SourceFile;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

use crate::api::mig::m5j_encoder::{
    append_uint32s, bool_to_byte, encode_diagnostic_directives, encode_file_references,
    encode_span_map, encode_string_array, msgpack_write_array_header, msgpack_write_bool,
    msgpack_write_string, msgpack_write_uint, FileReference, MappedDiagnosticDirective,
    NODE_DATA_TYPE_CHILDREN, NODE_DATA_TYPE_EXTENDED_DATA, NODE_DATA_TYPE_STRING,
    NO_STRUCTURED_DATA, PositionMap, SpanMapSegment,
};
use crate::api::mig::m5k_2::StringTable;

pub const KIND_LAST_UNARY_OPERATOR_MAX: u32 = 0x3f;

const KIND_LAST_UNARY_OPERATOR: u32 = SyntaxKind::TildeToken as u32;

pub fn init() { ::tsox_core::fntrace::enter("init"); 
    if KIND_LAST_UNARY_OPERATOR > KIND_LAST_UNARY_OPERATOR_MAX {
        panic!(
            "KindLastUnaryOperator ({}) exceeds the 6-bit commonData capacity (max {})",
            KIND_LAST_UNARY_OPERATOR, KIND_LAST_UNARY_OPERATOR_MAX
        );
    }
}

pub fn get_node_data(
    node: &Arc<Node>,
    strs: &mut StringTable,
    position_map: &PositionMap,
    extended_data: &mut Vec<u8>,
    structured_data: &mut Vec<u8>,
) -> u32 { ::tsox_core::fntrace::enter("get_node_data"); 
    let t = get_node_data_type(node);
    match t {
        NODE_DATA_TYPE_CHILDREN => {
            t | get_node_common_data(node) | (get_children_property_mask(node) as u32)
        }
        NODE_DATA_TYPE_STRING => t | get_node_common_data(node) | record_node_strings(node, strs),
        NODE_DATA_TYPE_EXTENDED_DATA => {
            t | get_node_common_data(node)
                | record_extended_data(node, strs, position_map, extended_data, structured_data)
        }
        _ => panic!("unreachable"),
    }
}

pub fn has_modifiers(modifiers: Option<&ModifierList>) -> bool { ::tsox_core::fntrace::enter("has_modifiers"); 
    match modifiers {
        Some(modifiers) => !modifiers.is_empty(),
        None => false,
    }
}

pub fn msgpack_write_array_header_go(buf: &mut Vec<u8>, length: usize) { ::tsox_core::fntrace::enter("msgpack_write_array_header_go"); 
    msgpack_write_array_header(buf, length)
}

pub fn msgpack_write_bool_go(buf: &mut Vec<u8>, value: bool) { ::tsox_core::fntrace::enter("msgpack_write_bool_go"); 
    msgpack_write_bool(buf, value)
}

pub fn msgpack_write_string_go(buf: &mut Vec<u8>, s: &str) { ::tsox_core::fntrace::enter("msgpack_write_string_go"); 
    msgpack_write_string(buf, s)
}

pub fn msgpack_write_uint_go(buf: &mut Vec<u8>, value: u32) { ::tsox_core::fntrace::enter("msgpack_write_uint_go"); 
    msgpack_write_uint(buf, value)
}

pub fn record_extended_data_source_file(
    node: &Arc<Node>,
    strs: &mut StringTable,
    position_map: &PositionMap,
    extended_data: &mut Vec<u8>,
    structured_data: &mut Vec<u8>,
) { ::tsox_core::fntrace::enter("record_extended_data_source_file"); 
    let sf = node_source_file_of(node);
    let original_text = source_file_original_text(&sf);
    let text_index = strs.add(&sf.text, node.kind, node.pos(), node.end());
    let original_text_index = if original_text != sf.text {
        strs.add(&original_text, SyntaxKind::Unknown, 0, 0)
    } else {
        text_index
    };
    let file_name_index = strs.add(&sf.file_name, SyntaxKind::Unknown, 0, 0);
    let path = source_file_path(&sf);
    let path_index = strs.add(&path, SyntaxKind::Unknown, 0, 0);
    let referenced_files_offset = encode_file_references(
        &to_encoder_file_references(&sf.referenced_files),
        position_map,
        structured_data,
    );
    let type_ref_directives_offset = encode_file_references(
        &to_encoder_file_references(&sf.type_reference_directives),
        position_map,
        structured_data,
    );
    let lib_ref_directives_offset = encode_file_references(
        &to_encoder_file_references(&sf.lib_reference_directives),
        position_map,
        structured_data,
    );
    let mut span_map_offset = NO_STRUCTURED_DATA;
    if let Some(span_map) = source_file_span_map(&sf) {
        span_map_offset = encode_span_map(
            &span_map,
            position_map,
            &ast_compute_position_map(&original_text),
            structured_data,
        );
    }
    let supplemental_file_names: Vec<String> = sf
        .supplemental_source_files()
        .iter()
        .map(|file| file.file_name.clone())
        .collect();
    let supplemental_file_names_offset =
        encode_string_array(&supplemental_file_names, structured_data);
    let mut canonical_file_name_index = NO_STRUCTURED_DATA;
    if let Some(canonical) = source_file_canonical(&sf) {
        canonical_file_name_index = strs.add(&canonical.file_name, SyntaxKind::Unknown, 0, 0);
    }
    let content_mapper = source_file_content_mapper(&sf);
    let mut content_mapper_index = NO_STRUCTURED_DATA;
    if !content_mapper.is_empty() {
        content_mapper_index = strs.add(&content_mapper, SyntaxKind::Unknown, 0, 0);
    }
    let virtual_file_name = source_file_virtual_file_name(&sf);
    let mut virtual_file_name_index = NO_STRUCTURED_DATA;
    if !virtual_file_name.is_empty() {
        virtual_file_name_index = strs.add(&virtual_file_name, SyntaxKind::Unknown, 0, 0);
    }
    let diagnostic_directives = source_file_diagnostic_directives(&sf);
    let diagnostic_directives_offset = encode_diagnostic_directives(
        &diagnostic_directives,
        position_map,
        &ast_compute_position_map(&original_text),
        structured_data,
    );
    let values: [u32; 19] = [
        text_index,
        file_name_index,
        path_index,
        sf.language_variant as u32,
        sf.script_kind as u32,
        referenced_files_offset,
        type_ref_directives_offset,
        lib_ref_directives_offset,
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
        0,
        original_text_index,
        span_map_offset,
        supplemental_file_names_offset,
        canonical_file_name_index,
        content_mapper_index,
        virtual_file_name_index,
        diagnostic_directives_offset,
    ];
    extended_data.extend_from_slice(&append_uint32s(Vec::new(), &values));
}

pub fn record_extended_data_template_head(
    node: &Node,
    strs: &mut StringTable,
    position_map: &PositionMap,
    extended_data: &mut Vec<u8>,
    structured_data: &mut Vec<u8>,
) { ::tsox_core::fntrace::enter("record_extended_data_template_head"); 
    let _ = (position_map, structured_data);
    let text_index = strs.add(node.text(), node.kind, node.pos(), node.end());
    let raw_text_index = strs.add(
        &node_template_head_raw_text(node),
        node.kind,
        node.pos(),
        node.end(),
    );
    extended_data.extend_from_slice(&append_uint32s(
        Vec::new(),
        &[text_index, raw_text_index, node_template_flags(node)],
    ));
}

pub fn record_extended_data_template_middle(
    node: &Node,
    strs: &mut StringTable,
    position_map: &PositionMap,
    extended_data: &mut Vec<u8>,
    structured_data: &mut Vec<u8>,
) { ::tsox_core::fntrace::enter("record_extended_data_template_middle"); 
    let _ = (position_map, structured_data);
    let text_index = strs.add(node.text(), node.kind, node.pos(), node.end());
    let raw_text_index = strs.add(
        &node_template_middle_raw_text(node),
        node.kind,
        node.pos(),
        node.end(),
    );
    extended_data.extend_from_slice(&append_uint32s(
        Vec::new(),
        &[text_index, raw_text_index, node_template_flags(node)],
    ));
}

pub fn record_extended_data_template_tail(
    node: &Node,
    strs: &mut StringTable,
    position_map: &PositionMap,
    extended_data: &mut Vec<u8>,
    structured_data: &mut Vec<u8>,
) { ::tsox_core::fntrace::enter("record_extended_data_template_tail"); 
    let _ = (position_map, structured_data);
    let text_index = strs.add(node.text(), node.kind, node.pos(), node.end());
    let raw_text_index = strs.add(
        &node_template_tail_raw_text(node),
        node.kind,
        node.pos(),
        node.end(),
    );
    extended_data.extend_from_slice(&append_uint32s(
        Vec::new(),
        &[text_index, raw_text_index, node_template_flags(node)],
    ));
}

pub fn record_extended_data_string_literal(
    node: &Node,
    strs: &mut StringTable,
    extended_data: &mut Vec<u8>,
) { ::tsox_core::fntrace::enter("record_extended_data_string_literal"); 
    let text_index = strs.add(node.text(), node.kind, node.pos(), node.end());
    extended_data.extend_from_slice(&append_uint32s(
        Vec::new(),
        &[text_index, node_token_flags(node)],
    ));
}

pub fn record_extended_data_numeric_literal(
    node: &Node,
    strs: &mut StringTable,
    extended_data: &mut Vec<u8>,
) { ::tsox_core::fntrace::enter("record_extended_data_numeric_literal"); 
    let text_index = strs.add(node.text(), node.kind, node.pos(), node.end());
    extended_data.extend_from_slice(&append_uint32s(
        Vec::new(),
        &[text_index, node_token_flags(node)],
    ));
}

pub fn record_extended_data_big_int_literal(
    node: &Node,
    strs: &mut StringTable,
    extended_data: &mut Vec<u8>,
) { ::tsox_core::fntrace::enter("record_extended_data_big_int_literal"); 
    let text_index = strs.add(node.text(), node.kind, node.pos(), node.end());
    extended_data.extend_from_slice(&append_uint32s(
        Vec::new(),
        &[text_index, node_token_flags(node)],
    ));
}

pub fn record_extended_data_regular_expression_literal(
    node: &Node,
    strs: &mut StringTable,
    extended_data: &mut Vec<u8>,
) { ::tsox_core::fntrace::enter("record_extended_data_regular_expression_literal"); 
    let text_index = strs.add(node.text(), node.kind, node.pos(), node.end());
    extended_data.extend_from_slice(&append_uint32s(
        Vec::new(),
        &[text_index, node_token_flags(node)],
    ));
}

pub fn record_extended_data_no_substitution_template_literal(
    node: &Node,
    strs: &mut StringTable,
    extended_data: &mut Vec<u8>,
) { ::tsox_core::fntrace::enter("record_extended_data_no_substitution_template_literal"); 
    let text_index = strs.add(node.text(), node.kind, node.pos(), node.end());
    extended_data.extend_from_slice(&append_uint32s(
        Vec::new(),
        &[text_index, node_template_flags(node)],
    ));
}

fn node_source_file_of(node: &Arc<Node>) -> Arc<SourceFile> { ::tsox_core::fntrace::enter("node_source_file_of"); 
    tsox_checker::checker::mig::m3a_2::r31k1_defs::source_file_of_node(node)
        .expect("source file not registered")
}

fn to_encoder_file_references(
    refs: &[tsox_frontend::ast::node_source_file::FileReference],
) -> Vec<crate::api::mig::m5j_encoder::FileReference> { ::tsox_core::fntrace::enter("to_encoder_file_references"); 
    refs.iter()
        .map(|r| crate::api::mig::m5j_encoder::FileReference {
            pos: r.range.pos as usize,
            end: r.range.end as usize,
            file_name: r.file_name.clone(),
            resolution_mode: r.resolution_mode as u32,
            preserve: r.preserve,
        })
        .collect()
}

fn source_file_content_mapper_info(
    sf: &SourceFile,
) -> Option<tsox_compile::mig::m3l_cm_2::ContentMapperSourceFileInfo> { ::tsox_core::fntrace::enter("source_file_content_mapper_info"); 
    tsox_compile::mig::m3l_cm_2::content_mapper_source_file_info(&sf.file_name)
}

fn source_file_original_text(sf: &SourceFile) -> String { ::tsox_core::fntrace::enter("source_file_original_text"); 
    match source_file_content_mapper_info(sf) {
        Some(info) if !info.content_mapper.is_empty() => info.original_text,
        _ => sf.text.clone(),
    }
}

fn source_file_path(sf: &SourceFile) -> String { ::tsox_core::fntrace::enter("source_file_path"); 
    tsox_frontend::ast::mig::m3b_2::path(sf)
}

fn source_file_content_mapper(sf: &SourceFile) -> String { ::tsox_core::fntrace::enter("source_file_content_mapper"); 
    source_file_content_mapper_info(sf)
        .map(|info| info.content_mapper)
        .unwrap_or_default()
}

fn source_file_virtual_file_name(sf: &SourceFile) -> String { ::tsox_core::fntrace::enter("source_file_virtual_file_name"); 
    source_file_content_mapper_info(sf)
        .map(|info| info.virtual_file_name)
        .unwrap_or_default()
}

fn source_file_canonical(sf: &SourceFile) -> Option<Arc<SourceFile>> { ::tsox_core::fntrace::enter("source_file_canonical"); 
    source_file_content_mapper_info(sf).and_then(|info| info.canonical_source_file)
}

fn source_file_span_map(_sf: &SourceFile) -> Option<Vec<SpanMapSegment>> { ::tsox_core::fntrace::enter("source_file_span_map"); 
    None
}

fn source_file_diagnostic_directives(
    sf: &SourceFile,
) -> Vec<crate::api::mig::m5j_encoder::MappedDiagnosticDirective> { ::tsox_core::fntrace::enter("source_file_diagnostic_directives"); 
    source_file_content_mapper_info(sf)
        .map(|info| {
            info.diagnostic_directives
                .iter()
                .map(|d| crate::api::mig::m5j_encoder::MappedDiagnosticDirective {
                    original_range_pos: d.original_range.pos as usize,
                    original_range_end: d.original_range.end as usize,
                    virtual_range_pos: d.virtual_range.pos as usize,
                    virtual_range_end: d.virtual_range.end as usize,
                    policy: d.policy.0 as u32,
                    unused_code: d.unused_code as u32,
                })
                .collect()
        })
        .unwrap_or_default()
}

fn ast_compute_position_map(_text: &str) -> PositionMap { ::tsox_core::fntrace::enter("ast_compute_position_map"); 
    PositionMap {}
}

fn node_template_head_raw_text(node: &Node) -> String { ::tsox_core::fntrace::enter("node_template_head_raw_text"); 
    match &node.data {
        NodeData::TemplateHead(d) => d.raw_text.clone(),
        _ => panic!("node is not a TemplateHead"),
    }
}

fn node_template_middle_raw_text(node: &Node) -> String { ::tsox_core::fntrace::enter("node_template_middle_raw_text"); 
    match &node.data {
        NodeData::TemplateMiddle(d) => d.raw_text.clone(),
        _ => panic!("node is not a TemplateMiddle"),
    }
}

fn node_template_tail_raw_text(node: &Node) -> String { ::tsox_core::fntrace::enter("node_template_tail_raw_text"); 
    match &node.data {
        NodeData::TemplateTail(d) => d.raw_text.clone(),
        _ => panic!("node is not a TemplateTail"),
    }
}

fn node_template_flags(node: &Node) -> u32 { ::tsox_core::fntrace::enter("node_template_flags"); 
    match &node.data {
        NodeData::TemplateHead(d) => d.template_flags,
        NodeData::TemplateMiddle(d) => d.template_flags,
        NodeData::TemplateTail(d) => d.template_flags,
        NodeData::NoSubstitutionTemplateLiteral(d) => d.template_flags,
        _ => panic!("node has no template flags"),
    }
}

fn node_token_flags(node: &Node) -> u32 { ::tsox_core::fntrace::enter("node_token_flags"); 
    match &node.data {
        NodeData::StringLiteral(d) => d.token_flags,
        NodeData::NumericLiteral(d) => d.token_flags,
        NodeData::BigIntLiteral(d) => d.token_flags,
        NodeData::RegularExpressionLiteral(d) => d.token_flags,
        _ => panic!("node has no token flags"),
    }
}

pub fn get_node_data_type(node: &Node) -> u32 { ::tsox_core::fntrace::enter("get_node_data_type"); 
    match node.kind {
        SyntaxKind::Identifier
        | SyntaxKind::PrivateIdentifier
        | SyntaxKind::JsxText
        | SyntaxKind::JSDocText
        | SyntaxKind::JSDocLink
        | SyntaxKind::JSDocLinkPlain
        | SyntaxKind::JSDocLinkCode => NODE_DATA_TYPE_STRING,
        SyntaxKind::StringLiteral
        | SyntaxKind::NumericLiteral
        | SyntaxKind::BigIntLiteral
        | SyntaxKind::RegularExpressionLiteral
        | SyntaxKind::NoSubstitutionTemplateLiteral
        | SyntaxKind::TemplateHead
        | SyntaxKind::TemplateMiddle
        | SyntaxKind::TemplateTail
        | SyntaxKind::SourceFile => NODE_DATA_TYPE_EXTENDED_DATA,
        _ => NODE_DATA_TYPE_CHILDREN,
    }
}

pub fn get_node_common_data(_node: &Node) -> u32 { ::tsox_core::fntrace::enter("get_node_common_data"); 
    0
}

pub fn get_children_property_mask(_node: &Node) -> u8 { ::tsox_core::fntrace::enter("get_children_property_mask"); 
    0
}

pub fn record_node_strings(_node: &Node, _strs: &mut StringTable) -> u32 { ::tsox_core::fntrace::enter("record_node_strings"); 
    0
}

pub fn record_extended_data(
    node: &Arc<Node>,
    strs: &mut StringTable,
    position_map: &PositionMap,
    extended_data: &mut Vec<u8>,
    structured_data: &mut Vec<u8>,
) -> u32 { ::tsox_core::fntrace::enter("record_extended_data"); 
    match node.kind {
        SyntaxKind::SourceFile => {
            record_extended_data_source_file(node, strs, position_map, extended_data, structured_data);
            0
        }
        SyntaxKind::TemplateHead => {
            record_extended_data_template_head(node, strs, position_map, extended_data, structured_data);
            0
        }
        SyntaxKind::TemplateMiddle => {
            record_extended_data_template_middle(node, strs, position_map, extended_data, structured_data);
            0
        }
        SyntaxKind::TemplateTail => {
            record_extended_data_template_tail(node, strs, position_map, extended_data, structured_data);
            0
        }
        SyntaxKind::StringLiteral => {
            record_extended_data_string_literal(node, strs, extended_data);
            0
        }
        SyntaxKind::NumericLiteral => {
            record_extended_data_numeric_literal(node, strs, extended_data);
            0
        }
        SyntaxKind::BigIntLiteral => {
            record_extended_data_big_int_literal(node, strs, extended_data);
            0
        }
        SyntaxKind::RegularExpressionLiteral => {
            record_extended_data_regular_expression_literal(node, strs, extended_data);
            0
        }
        SyntaxKind::NoSubstitutionTemplateLiteral => {
            record_extended_data_no_substitution_template_literal(node, strs, extended_data);
            0
        }
        _ => 0,
    }
}
