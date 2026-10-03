#![allow(unused_imports, dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use tsox_frontend::ast::node::{ModifierList, Node, NodeList, SourceFile};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

pub const NODE_OFFSET_KIND: usize = 0;
pub const NODE_OFFSET_POS: usize = 4;
pub const NODE_OFFSET_END: usize = 8;
pub const NODE_OFFSET_NEXT: usize = 12;
pub const NODE_OFFSET_PARENT: usize = 16;
pub const NODE_OFFSET_DATA: usize = 20;
pub const NODE_OFFSET_FLAGS: usize = 24;
pub const NODE_SIZE: usize = 28;

pub const NODE_DATA_TYPE_CHILDREN: u32 = 0b00 << 30;
pub const NODE_DATA_TYPE_STRING: u32 = 0b01 << 30;
pub const NODE_DATA_TYPE_EXTENDED_DATA: u32 = 0b10 << 30;
pub const NODE_DATA_TYPE_MASK: u32 = 0xc0_00_00_00;
pub const NODE_DATA_CHILD_MASK: u32 = 0x00_00_00_ff;
pub const NODE_DATA_STRING_INDEX_MASK: u32 = 0x00_ff_ff_ff;
pub const SYNTAX_KIND_NODE_LIST: u32 = u32::MAX;

pub const HEADER_OFFSET_METADATA: usize = 0;
pub const HEADER_OFFSET_HASH_LO0: usize = 4;
pub const HEADER_OFFSET_HASH_LO1: usize = 8;
pub const HEADER_OFFSET_HASH_HI0: usize = 12;
pub const HEADER_OFFSET_HASH_HI1: usize = 16;
pub const HEADER_OFFSET_PARSE_OPTIONS: usize = 20;
pub const HEADER_OFFSET_STRING_OFFSETS: usize = 24;
pub const HEADER_OFFSET_STRING_DATA: usize = 28;
pub const HEADER_OFFSET_EXTENDED_DATA: usize = 32;
pub const HEADER_OFFSET_STRUCTURED_DATA: usize = 36;
pub const HEADER_OFFSET_NODES: usize = 40;
pub const HEADER_SIZE: usize = 44;
pub const PROTOCOL_VERSION: u8 = 8;

pub const NO_STRUCTURED_DATA: u32 = 0xFFFFFFFF;

pub struct NodeIndexTable {
    pub nodes: Vec<Option<Arc<Node>>>,
    sorted_idx: std::sync::OnceLock<Vec<u32>>,
}

impl NodeIndexTable {
    pub fn get_index(&self, node: &Node) -> u32 { ::tsox_core::fntrace::enter("get_index"); 
        let sorted_idx = self.sorted_idx.get_or_init(|| {
            let mut idx: Vec<u32> = self
                .nodes
                .iter()
                .enumerate()
                .filter(|(_, n)| n.is_some())
                .map(|(i, _)| i as u32)
                .collect();
            idx.sort_by_key(|&i| ast_get_node_id(self.nodes[i as usize].as_ref().unwrap()));
            idx
        });
        let target = ast_get_node_id(node);
        let result = sorted_idx.binary_search_by(|&el| {
            ast_get_node_id(self.nodes[el as usize].as_ref().unwrap()).cmp(&target)
        });
        match result {
            Ok(i) => sorted_idx[i],
            Err(_) => 0,
        }
    }
}

fn ast_get_node_id(_node: &Node) -> u64 { ::tsox_core::fntrace::enter("ast_get_node_id"); 
    0
}

pub fn source_file_hash(source_file: &SourceFile) -> String { ::tsox_core::fntrace::enter("source_file_hash"); 
    let h = tsox_lsp::project::overlay_fs::hash_string_128(&source_file.text);
    format!("{:016x}{:016x}", h.hi, h.lo)
}

pub fn encode_parse_options(jsx: bool, force: bool) -> u32 { ::tsox_core::fntrace::enter("encode_parse_options"); 
    let mut bits: u32 = 0;
    if jsx {
        bits |= 1;
    }
    if force {
        bits |= 2;
    }
    bits
}

pub fn build_node_index_table(source_file: &SourceFile) -> NodeIndexTable { ::tsox_core::fntrace::enter("build_node_index_table"); 
    let mut node_table: Vec<Option<Arc<Node>>> = vec![None];
    node_table.push(Some(Arc::clone(&source_file.node)));
    NodeIndexTable {
        nodes: node_table,
        sorted_idx: std::sync::OnceLock::new(),
    }
}

pub fn get_node_index_table(source_file: &SourceFile) -> NodeIndexTable { ::tsox_core::fntrace::enter("get_node_index_table"); 
    build_node_index_table(source_file)
}

pub fn encode_source_file(source_file: &SourceFile) -> Result<(Vec<u8>, NodeIndexTable), String> { ::tsox_core::fntrace::enter("encode_source_file"); 
    encode_tree(&source_file.node, Some(source_file))
}

pub fn encode_node(node: &Arc<Node>, source_file: &SourceFile) -> Result<(Vec<u8>, NodeIndexTable), String> { ::tsox_core::fntrace::enter("encode_node"); 
    encode_tree(node, Some(source_file))
}

pub fn encode_tree(root_node: &Arc<Node>, source_file: Option<&SourceFile>) -> Result<(Vec<u8>, NodeIndexTable), String> { ::tsox_core::fntrace::enter("encode_tree"); 
    let mut nodes: Vec<u8> = Vec::new();
    let mut extended_data: Vec<u8> = Vec::new();
    let mut structured_data: Vec<u8> = Vec::new();
    let mut node_table: Vec<Option<Arc<Node>>> = vec![None];
    let mut node_count: u32 = 0;

    nodes = append_uint32s(nodes, &[0, 0, 0, 0, 0, 0, 0]);
    node_count += 1;
    node_table.push(Some(root_node.clone()));
    nodes = append_uint32s(
        nodes,
        &[
            root_node.kind as u32,
            utf16_of(root_node.pos()),
            utf16_of(root_node.end()),
            0,
            0,
            get_node_data(root_node, &mut extended_data, &mut structured_data),
            node_flags_to_u32(&root_node.flags),
        ],
    );

    let mut hash_lo: u64 = 0;
    let mut hash_hi: u64 = 0;
    let mut parse_opts: u32 = 0;
    if let Some(source_file) = source_file {
        let h = tsox_lsp::project::overlay_fs::hash_string_128(&source_file.text);
        hash_lo = h.lo;
        hash_hi = h.hi;
        parse_opts = encode_parse_options(false, false);
        let imports_offset = encode_node_index_array(&[], &mut HashMap::new(), &mut structured_data);
        let module_augmentations_offset = encode_module_augmentations(&[], &mut HashMap::new(), &mut structured_data);
        let ambient_module_names_offset = encode_string_array(&[], &mut structured_data);
        patch_u32(&mut extended_data, 32, imports_offset);
        patch_u32(&mut extended_data, 36, module_augmentations_offset);
        patch_u32(&mut extended_data, 40, ambient_module_names_offset);
        patch_u32(&mut extended_data, 44, 0);
    }

    let metadata: u32 = (PROTOCOL_VERSION as u32) << 24;
    let header: Vec<u32> = vec![
        metadata,
        hash_lo as u32,
        (hash_lo >> 32) as u32,
        hash_hi as u32,
        (hash_hi >> 32) as u32,
        parse_opts,
        HEADER_SIZE as u32,
        (HEADER_SIZE + 0) as u32,
        0,
        0,
        nodes.len() as u32,
    ];
    let mut out = append_uint32s(Vec::new(), &header);
    out.extend_from_slice(&extended_data);
    out.extend_from_slice(&structured_data);
    out.extend_from_slice(&nodes);
    Ok((out, NodeIndexTable { nodes: node_table, sorted_idx: std::sync::OnceLock::new() }))
}

fn utf16_of(pos: usize) -> u32 { ::tsox_core::fntrace::enter("utf16_of"); 
    pos as u32
}

fn node_flags_to_u32(_flags: &tsox_frontend::ast::node_flags::NodeFlags) -> u32 { ::tsox_core::fntrace::enter("node_flags_to_u32"); 
    0
}

fn patch_u32(buf: &mut [u8], offset: usize, value: u32) { ::tsox_core::fntrace::enter("patch_u32"); 
    if offset + 4 <= buf.len() {
        buf[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
}

pub fn get_node_data(
    node: &Node,
    extended_data: &mut Vec<u8>,
    structured_data: &mut Vec<u8>,
) -> u32 { ::tsox_core::fntrace::enter("get_node_data"); 
    let _ = (extended_data, structured_data);
    node.kind as u32
}

pub fn append_uint32s(mut buf: Vec<u8>, values: &[u32]) -> Vec<u8> { ::tsox_core::fntrace::enter("append_uint32s"); 
    for value in values {
        buf.extend_from_slice(&value.to_le_bytes());
    }
    buf
}

pub fn bool_to_byte(b: bool) -> u8 { ::tsox_core::fntrace::enter("bool_to_byte"); 
    if b {
        1
    } else {
        0
    }
}

pub fn encode_file_references(
    refs: &[FileReference],
    position_map: &PositionMap,
    buf: &mut Vec<u8>,
) -> u32 { ::tsox_core::fntrace::enter("encode_file_references"); 
    if refs.is_empty() {
        return NO_STRUCTURED_DATA;
    }
    let offset = buf.len() as u32;
    msgpack_write_array_header(buf, refs.len());
    for r in refs {
        msgpack_write_array_header(buf, 5);
        msgpack_write_uint(buf, position_map.utf8_to_utf16(r.pos));
        msgpack_write_uint(buf, position_map.utf8_to_utf16(r.end));
        msgpack_write_string(buf, &r.file_name);
        msgpack_write_uint(buf, r.resolution_mode);
        msgpack_write_bool(buf, r.preserve);
    }
    offset
}

pub struct FileReference {
    pub pos: usize,
    pub end: usize,
    pub file_name: String,
    pub resolution_mode: u32,
    pub preserve: bool,
}

pub struct PositionMap;

impl PositionMap {
    pub fn utf8_to_utf16(&self, pos: usize) -> u32 { ::tsox_core::fntrace::enter("utf8_to_utf16"); 
        pos as u32
    }
}

pub fn encode_node_index_array(
    nodes: &[Arc<Node>],
    index_map: &mut HashMap<*const Node, u32>,
    buf: &mut Vec<u8>,
) -> u32 { ::tsox_core::fntrace::enter("encode_node_index_array"); 
    if nodes.is_empty() {
        return NO_STRUCTURED_DATA;
    }
    let offset = buf.len() as u32;
    msgpack_write_array_header(buf, nodes.len());
    for node in nodes {
        msgpack_write_uint(buf, *index_map.get(&Arc::as_ptr(node)).unwrap_or(&0));
    }
    offset
}

pub fn encode_module_augmentations(
    nodes: &[Arc<Node>],
    index_map: &mut HashMap<*const Node, u32>,
    buf: &mut Vec<u8>,
) -> u32 { ::tsox_core::fntrace::enter("encode_module_augmentations"); 
    encode_node_index_array(nodes, index_map, buf)
}

pub fn encode_string_array(strs: &[String], buf: &mut Vec<u8>) -> u32 { ::tsox_core::fntrace::enter("encode_string_array"); 
    if strs.is_empty() {
        return NO_STRUCTURED_DATA;
    }
    let offset = buf.len() as u32;
    msgpack_write_array_header(buf, strs.len());
    for s in strs {
        msgpack_write_string(buf, s);
    }
    offset
}

pub fn encode_span_map(
    segments: &[SpanMapSegment],
    virtual_positions: &PositionMap,
    original_positions: &PositionMap,
    buf: &mut Vec<u8>,
) -> u32 { ::tsox_core::fntrace::enter("encode_span_map"); 
    if segments.is_empty() {
        return NO_STRUCTURED_DATA;
    }
    let offset = buf.len() as u32;
    msgpack_write_array_header(buf, segments.len());
    for segment in segments {
        let tuple_length = if segment.features == SPAN_MAP_FEATURE_ALL { 5 } else { 6 };
        msgpack_write_array_header(buf, tuple_length);
        let virtual_start = virtual_positions.utf8_to_utf16(segment.virtual_start);
        let virtual_end = virtual_positions.utf8_to_utf16(segment.virtual_end);
        let original_start = original_positions.utf8_to_utf16(segment.original_start);
        let original_end = original_positions.utf8_to_utf16(segment.original_end);
        msgpack_write_uint(buf, virtual_start);
        msgpack_write_uint(buf, virtual_end - virtual_start);
        msgpack_write_uint(buf, original_start);
        msgpack_write_uint(buf, original_end - original_start);
        msgpack_write_uint(buf, segment.kind);
        if tuple_length == 6 {
            msgpack_write_uint(buf, segment.features);
        }
    }
    offset
}

pub struct SpanMapSegment {
    pub virtual_start: usize,
    pub virtual_end: usize,
    pub original_start: usize,
    pub original_end: usize,
    pub kind: u32,
    pub features: u32,
}

pub const SPAN_MAP_FEATURE_ALL: u32 = u32::MAX;

pub fn encode_diagnostic_directives(
    directives: &[MappedDiagnosticDirective],
    virtual_positions: &PositionMap,
    original_positions: &PositionMap,
    buf: &mut Vec<u8>,
) -> u32 { ::tsox_core::fntrace::enter("encode_diagnostic_directives"); 
    if directives.is_empty() {
        return NO_STRUCTURED_DATA;
    }
    let offset = buf.len() as u32;
    msgpack_write_array_header(buf, directives.len());
    for directive in directives {
        msgpack_write_array_header(buf, 6);
        let original_start = original_positions.utf8_to_utf16(directive.original_range_pos);
        let original_end = original_positions.utf8_to_utf16(directive.original_range_end);
        let virtual_start = virtual_positions.utf8_to_utf16(directive.virtual_range_pos);
        let virtual_end = virtual_positions.utf8_to_utf16(directive.virtual_range_end);
        msgpack_write_uint(buf, original_start);
        msgpack_write_uint(buf, original_end - original_start);
        msgpack_write_uint(buf, virtual_start);
        msgpack_write_uint(buf, virtual_end - virtual_start);
        msgpack_write_uint(buf, directive.policy);
        msgpack_write_uint(buf, directive.unused_code);
    }
    offset
}

pub struct MappedDiagnosticDirective {
    pub original_range_pos: usize,
    pub original_range_end: usize,
    pub virtual_range_pos: usize,
    pub virtual_range_end: usize,
    pub policy: u32,
    pub unused_code: u32,
}

pub fn msgpack_write_array_header(buf: &mut Vec<u8>, length: usize) { ::tsox_core::fntrace::enter("msgpack_write_array_header"); 
    if length <= 0x0f {
        buf.push(0x90 | length as u8);
    } else if length <= 0xffff {
        buf.push(0xdc);
        buf.push((length >> 8) as u8);
        buf.push(length as u8);
    } else {
        buf.push(0xdd);
        buf.push((length >> 24) as u8);
        buf.push((length >> 16) as u8);
        buf.push((length >> 8) as u8);
        buf.push(length as u8);
    }
}

pub fn msgpack_write_uint(buf: &mut Vec<u8>, value: u32) { ::tsox_core::fntrace::enter("msgpack_write_uint"); 
    if value <= 0x7f {
        buf.push(value as u8);
    } else if value <= 0xff {
        buf.push(0xcc);
        buf.push(value as u8);
    } else if value <= 0xffff {
        buf.push(0xcd);
        buf.push((value >> 8) as u8);
        buf.push(value as u8);
    } else {
        buf.push(0xce);
        buf.push((value >> 24) as u8);
        buf.push((value >> 16) as u8);
        buf.push((value >> 8) as u8);
        buf.push(value as u8);
    }
}

pub fn msgpack_write_string(buf: &mut Vec<u8>, s: &str) { ::tsox_core::fntrace::enter("msgpack_write_string"); 
    let n = s.len();
    if n <= 0x1f {
        buf.push(0xa0 | n as u8);
    } else if n <= 0xff {
        buf.push(0xd9);
        buf.push(n as u8);
    } else if n <= 0xffff {
        buf.push(0xda);
        buf.push((n >> 8) as u8);
        buf.push(n as u8);
    } else {
        buf.push(0xdb);
        buf.push((n >> 24) as u8);
        buf.push((n >> 16) as u8);
        buf.push((n >> 8) as u8);
        buf.push(n as u8);
    }
    buf.extend_from_slice(s.as_bytes());
}

pub fn msgpack_write_bool(buf: &mut Vec<u8>, value: bool) { ::tsox_core::fntrace::enter("msgpack_write_bool"); 
    buf.push(if value { 0xc3 } else { 0xc2 });
}

pub fn get_node_common_data_synthetic_expression(_node: &Node) -> u32 { ::tsox_core::fntrace::enter("get_node_common_data_synthetic_expression"); 
    panic!("SyntheticExpression should never be encoded");
}
