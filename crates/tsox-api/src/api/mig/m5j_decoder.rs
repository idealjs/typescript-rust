#![allow(unused_imports, dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::node_node::Node;
use tsox_frontend::ast::node_node_list::{ModifierList, NodeList};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

use super::m5j_encoder::*;

pub struct AstDecoder {
    pub raw: Vec<u8>,
    pub str_table: u32,
    pub str_data: u32,
    pub ext_data: u32,
    pub node_off: u32,
    pub node_count: usize,
    pub child_buf: Vec<usize>,
    pub all_string_data: Vec<u8>,
    pub nodes: Vec<Option<Arc<Node>>>,
    pub node_lists: Vec<Option<NodeList>>,
}

pub fn decode_source_file(data: &[u8]) -> Result<Arc<Node>, String> { ::tsox_core::fntrace::enter("decode_source_file"); 
    let node = decode_nodes(data)?;
    if node.kind != SyntaxKind::SourceFile {
        return Err(format!("expected SourceFile root, got {:?}", node.kind));
    }
    Ok(node)
}

pub fn decode_nodes(data: &[u8]) -> Result<Arc<Node>, String> { ::tsox_core::fntrace::enter("decode_nodes"); 
    let mut d = new_ast_decoder(data)?;
    d.decode()
}

pub fn new_ast_decoder(data: &[u8]) -> Result<AstDecoder, String> { ::tsox_core::fntrace::enter("new_ast_decoder"); 
    if data.len() < HEADER_SIZE {
        return Err(format!("data too short for header: {} bytes", data.len()));
    }
    let version = data[HEADER_OFFSET_METADATA + 3];
    if version != PROTOCOL_VERSION {
        return Err(format!(
            "unsupported protocol version {} (expected {})",
            version, PROTOCOL_VERSION
        ));
    }
    let str_table = read_le32(data, HEADER_OFFSET_STRING_OFFSETS);
    let str_data = read_le32(data, HEADER_OFFSET_STRING_DATA);
    let ext_data = read_le32(data, HEADER_OFFSET_EXTENDED_DATA);
    let node_off = read_le32(data, HEADER_OFFSET_NODES);
    let data_len = data.len() as u32;
    if str_table > data_len || str_data > data_len || ext_data > data_len || node_off > data_len {
        return Err(format!(
            "invalid AST header offsets: offsets exceed data length ({})",
            data_len
        ));
    }
    if !(str_table <= str_data && str_data <= ext_data && ext_data <= node_off) {
        return Err(
            "invalid AST header offsets: expected strTable <= strData <= extData <= nodeOff".to_string(),
        );
    }
    let node_count = (data.len() - node_off as usize) / NODE_SIZE;
    Ok(AstDecoder {
        raw: data.to_vec(),
        str_table,
        str_data,
        ext_data,
        node_off,
        node_count,
        child_buf: Vec::new(),
        all_string_data: data[str_data as usize..].to_vec(),
        nodes: Vec::new(),
        node_lists: Vec::new(),
    })
}

impl AstDecoder {
    pub fn alloc_node_slice(&mut self, capacity: usize) -> Vec<Option<Arc<Node>>> { ::tsox_core::fntrace::enter("alloc_node_slice"); 
        Vec::with_capacity(capacity)
    }

    pub fn node_field(&self, i: usize, field: usize) -> u32 { ::tsox_core::fntrace::enter("node_field"); 
        read_le32(&self.raw, self.node_off as usize + i * NODE_SIZE + field)
    }

    pub fn get_string(&self, idx: u32) -> String { ::tsox_core::fntrace::enter("get_string"); 
        let off_base = self.str_table as usize + idx as usize * 4;
        let start = read_le32(&self.raw, off_base) as usize;
        let end = read_le32(&self.raw, off_base + 4) as usize;
        String::from_utf8_lossy(&self.all_string_data[start..end]).into_owned()
    }

    pub fn collect_children(&mut self, i: usize) -> Vec<usize> { ::tsox_core::fntrace::enter("collect_children"); 
        self.child_buf.clear();
        if i + 1 >= self.node_count {
            return std::mem::take(&mut self.child_buf);
        }
        let first_child = i + 1;
        if self.node_field(first_child, NODE_OFFSET_PARENT) != i as u32 {
            return std::mem::take(&mut self.child_buf);
        }
        self.child_buf.push(first_child);
        let mut next = self.node_field(first_child, NODE_OFFSET_NEXT) as usize;
        while next != 0 {
            self.child_buf.push(next);
            next = self.node_field(next, NODE_OFFSET_NEXT) as usize;
        }
        std::mem::take(&mut self.child_buf)
    }

    pub fn decode(&mut self) -> Result<Arc<Node>, String> { ::tsox_core::fntrace::enter("decode"); 
        if self.node_count < 2 {
            return Err("no nodes to decode".to_string());
        }
        self.nodes = vec![None; self.node_count];
        self.node_lists = vec![None; self.node_count];
        for i in (1..self.node_count).rev() {
            let kind = self.node_field(i, NODE_OFFSET_KIND);
            let _pos = self.node_field(i, NODE_OFFSET_POS);
            let _end = self.node_field(i, NODE_OFFSET_END);
            let data = self.node_field(i, NODE_OFFSET_DATA);
            let child_indices = self.collect_children(i);
            if kind == SYNTAX_KIND_NODE_LIST {
                let children = self.alloc_node_slice(child_indices.len());
                let _ = children;
                continue;
            }
            let node = self.create_node(kind, data, child_indices)?;
            self.nodes[i] = Some(node);
        }
        self.nodes
            .get(1)
            .cloned()
            .flatten()
            .ok_or_else(|| "missing root node".to_string())
    }

    pub fn get_modifier_list(&self, ci: usize) -> Option<ModifierList> { ::tsox_core::fntrace::enter("get_modifier_list"); 
        let nl = self.node_lists.get(ci)?.as_ref()?;
        Some(new_modifier_list_from(&nl.nodes))
    }

    pub fn node_at(&self, ci: usize) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("node_at"); 
        if ci == 0 {
            return None;
        }
        self.nodes.get(ci)?.clone()
    }

    pub fn node_list_at(&self, ci: usize) -> Option<NodeList> { ::tsox_core::fntrace::enter("node_list_at"); 
        if ci == 0 {
            return None;
        }
        self.node_lists.get(ci)?.clone()
    }

    pub fn modifier_list_at(&self, ci: usize) -> Option<ModifierList> { ::tsox_core::fntrace::enter("modifier_list_at"); 
        if ci == 0 {
            return None;
        }
        self.get_modifier_list(ci)
    }

    pub fn create_node(&self, kind: u32, data: u32, child_indices: Vec<usize>) -> Result<Arc<Node>, String> { ::tsox_core::fntrace::enter("create_node"); 
        let data_type = data & NODE_DATA_TYPE_MASK;
        let _common_data = ((data >> 24) & 0x3f) as u8;
        match data_type {
            NODE_DATA_TYPE_STRING => Ok(self.decode_extended_data_string_literal(data)),
            NODE_DATA_TYPE_EXTENDED_DATA => self.decode_extended_data_source_file(data, child_indices),
            _ => self.decode_children_node(kind, data, child_indices),
        }
    }

    pub fn decode_extended_data_source_file(
        &self,
        data: u32,
        child_indices: Vec<usize>,
    ) -> Result<Arc<Node>, String> { ::tsox_core::fntrace::enter("decode_extended_data_source_file"); 
        let ext_off = self.ext_data as usize + (data & NODE_DATA_STRING_INDEX_MASK) as usize;
        let text_idx = read_le32(&self.raw, ext_off);
        let file_name_idx = read_le32(&self.raw, ext_off + 4);
        let path_idx = read_le32(&self.raw, ext_off + 8);
        let _text = self.get_string(text_idx);
        let _file_name = self.get_string(file_name_idx);
        let _path = self.get_string(path_idx);
        let _parse_opts = read_le32(&self.raw, HEADER_OFFSET_PARSE_OPTIONS);
        let mut stmts: Option<NodeList> = None;
        let mut end_of_file: Option<Arc<Node>> = None;
        for ci in child_indices {
            if self.node_field(ci, NODE_OFFSET_KIND) == SYNTAX_KIND_NODE_LIST {
                stmts = self.node_list_at(ci);
            } else if let Some(child) = self.node_at(ci) {
                if child.kind == SyntaxKind::EndOfFile {
                    end_of_file = Some(child);
                }
            }
        }
        if end_of_file.is_none() {
            end_of_file = Some(new_token_node(SyntaxKind::EndOfFile));
        }
        Ok(new_source_file_node(stmts, end_of_file))
    }

    pub fn decode_extended_data_template_head(&self, data: u32) -> Arc<Node> { ::tsox_core::fntrace::enter("decode_extended_data_template_head"); 
        let ext_off = self.ext_data as usize + (data & NODE_DATA_STRING_INDEX_MASK) as usize;
        let text_idx = read_le32(&self.raw, ext_off);
        let raw_text_idx = read_le32(&self.raw, ext_off + 4);
        let _flags = read_le32(&self.raw, ext_off + 8);
        new_string_like_node(SyntaxKind::TemplateHead, &self.get_string(text_idx), &self.get_string(raw_text_idx))
    }

    pub fn decode_extended_data_template_middle(&self, data: u32) -> Arc<Node> { ::tsox_core::fntrace::enter("decode_extended_data_template_middle"); 
        let ext_off = self.ext_data as usize + (data & NODE_DATA_STRING_INDEX_MASK) as usize;
        let text_idx = read_le32(&self.raw, ext_off);
        let raw_text_idx = read_le32(&self.raw, ext_off + 4);
        let _flags = read_le32(&self.raw, ext_off + 8);
        new_string_like_node(SyntaxKind::TemplateMiddle, &self.get_string(text_idx), &self.get_string(raw_text_idx))
    }

    pub fn decode_extended_data_template_tail(&self, data: u32) -> Arc<Node> { ::tsox_core::fntrace::enter("decode_extended_data_template_tail"); 
        let ext_off = self.ext_data as usize + (data & NODE_DATA_STRING_INDEX_MASK) as usize;
        let text_idx = read_le32(&self.raw, ext_off);
        let raw_text_idx = read_le32(&self.raw, ext_off + 4);
        let _flags = read_le32(&self.raw, ext_off + 8);
        new_string_like_node(SyntaxKind::TemplateTail, &self.get_string(text_idx), &self.get_string(raw_text_idx))
    }

    pub fn decode_extended_data_string_literal(&self, data: u32) -> Arc<Node> { ::tsox_core::fntrace::enter("decode_extended_data_string_literal"); 
        let ext_off = self.ext_data as usize + (data & NODE_DATA_STRING_INDEX_MASK) as usize;
        let text_idx = read_le32(&self.raw, ext_off);
        let _flags = read_le32(&self.raw, ext_off + 4);
        new_string_like_node(SyntaxKind::StringLiteral, &self.get_string(text_idx), "")
    }

    pub fn decode_extended_data_numeric_literal(&self, data: u32) -> Arc<Node> { ::tsox_core::fntrace::enter("decode_extended_data_numeric_literal"); 
        let ext_off = self.ext_data as usize + (data & NODE_DATA_STRING_INDEX_MASK) as usize;
        let text_idx = read_le32(&self.raw, ext_off);
        let _flags = read_le32(&self.raw, ext_off + 4);
        new_string_like_node(SyntaxKind::NumericLiteral, &self.get_string(text_idx), "")
    }

    pub fn decode_extended_data_big_int_literal(&self, data: u32) -> Arc<Node> { ::tsox_core::fntrace::enter("decode_extended_data_big_int_literal"); 
        let ext_off = self.ext_data as usize + (data & NODE_DATA_STRING_INDEX_MASK) as usize;
        let text_idx = read_le32(&self.raw, ext_off);
        let _flags = read_le32(&self.raw, ext_off + 4);
        new_string_like_node(SyntaxKind::BigIntLiteral, &self.get_string(text_idx), "")
    }

    pub fn decode_extended_data_regular_expression_literal(&self, data: u32) -> Arc<Node> { ::tsox_core::fntrace::enter("decode_extended_data_regular_expression_literal"); 
        let ext_off = self.ext_data as usize + (data & NODE_DATA_STRING_INDEX_MASK) as usize;
        let text_idx = read_le32(&self.raw, ext_off);
        let _flags = read_le32(&self.raw, ext_off + 4);
        new_string_like_node(SyntaxKind::RegularExpressionLiteral, &self.get_string(text_idx), "")
    }

    pub fn decode_extended_data_no_substitution_template_literal(&self, data: u32) -> Arc<Node> { ::tsox_core::fntrace::enter("decode_extended_data_no_substitution_template_literal"); 
        let ext_off = self.ext_data as usize + (data & NODE_DATA_STRING_INDEX_MASK) as usize;
        let text_idx = read_le32(&self.raw, ext_off);
        let _flags = read_le32(&self.raw, ext_off + 4);
        new_string_like_node(SyntaxKind::NoSubstitutionTemplateLiteral, &self.get_string(text_idx), "")
    }

    pub fn single_child(&self, child_indices: &[usize]) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("single_child"); 
        child_indices.first().and_then(|ci| self.nodes[*ci].clone())
    }

    pub fn single_node_list_child(&self, child_indices: &[usize]) -> Option<NodeList> { ::tsox_core::fntrace::enter("single_node_list_child"); 
        child_indices.first().and_then(|ci| self.node_lists[*ci].clone())
    }

    fn decode_children_node(&self, kind: u32, data: u32, child_indices: Vec<usize>) -> Result<Arc<Node>, String> { ::tsox_core::fntrace::enter("decode_children_node"); 
        let _ = (kind, data, child_indices);
        Ok(new_children_node(kind))
    }
}

pub struct ChildIterator {
    pub indices: Vec<usize>,
    pub pos: usize,
}

pub fn new_child_iter(indices: Vec<usize>) -> ChildIterator { ::tsox_core::fntrace::enter("new_child_iter"); 
    ChildIterator { indices, pos: 0 }
}

impl ChildIterator {
    pub fn next(&mut self) -> usize { ::tsox_core::fntrace::enter("next"); 
        if self.pos >= self.indices.len() {
            return 0;
        }
        let ci = self.indices[self.pos];
        self.pos += 1;
        ci
    }

    pub fn next_if(&mut self, mask: u8, bit: u8) -> usize { ::tsox_core::fntrace::enter("next_if"); 
        if mask & (1 << bit) == 0 {
            return 0;
        }
        self.next()
    }
}

pub fn read_le32(data: &[u8], offset: usize) -> u32 { ::tsox_core::fntrace::enter("read_le32"); 
    if offset + 4 > data.len() {
        return 0;
    }
    u32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])
}

pub fn decode_node_common_data_synthetic_expression(_common_data: u8) -> ! { ::tsox_core::fntrace::enter("decode_node_common_data_synthetic_expression"); 
    panic!("SyntheticExpression should never be decoded");
}

fn new_token_node(kind: SyntaxKind) -> Arc<Node> { ::tsox_core::fntrace::enter("new_token_node"); 
    Arc::new(Node::new(kind, tsox_frontend::ast::node_data_generated::NodeData::Token))
}

fn new_source_file_node(_stmts: Option<NodeList>, _end_of_file: Option<Arc<Node>>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_source_file_node"); 
    Arc::new(Node::new(SyntaxKind::SourceFile, tsox_frontend::ast::node_data_generated::NodeData::Token))
}

fn new_string_like_node(kind: SyntaxKind, _text: &str, _raw_text: &str) -> Arc<Node> { ::tsox_core::fntrace::enter("new_string_like_node"); 
    Arc::new(Node::new(kind, tsox_frontend::ast::node_data_generated::NodeData::Token))
}

fn new_children_node(kind: u32) -> Arc<Node> { ::tsox_core::fntrace::enter("new_children_node"); 
    let kind = unsafe { std::mem::transmute::<i16, SyntaxKind>(kind as i16) };
    Arc::new(Node::new(kind, tsox_frontend::ast::node_data_generated::NodeData::Token))
}

fn new_modifier_list_from(_nodes: &[Arc<Node>]) -> ModifierList { ::tsox_core::fntrace::enter("new_modifier_list_from"); 
    ModifierList::default()
}
