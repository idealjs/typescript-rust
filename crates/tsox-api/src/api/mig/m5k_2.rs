#![allow(unused_imports, dead_code)]

use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

use crate::api::mig::m5j_encoder::append_uint32s;

pub struct StringTable {
    file_text: String,
    other_strings: String,
    offsets: Vec<u32>,
}

pub fn new_string_table(file_text: String, string_count: usize) -> StringTable {
    StringTable {
        file_text,
        other_strings: String::new(),
        offsets: Vec::with_capacity(string_count * 2),
    }
}

impl StringTable {
    pub fn add(&mut self, text: &str, kind: SyntaxKind, pos: usize, end: usize) -> u32 {
        let index = self.offsets.len() as u32;
        if kind == SyntaxKind::SourceFile {
            self.offsets.push(pos as u32);
            self.offsets.push(end as u32);
            return index;
        }
        let length = text.len();
        if end > pos && end <= self.file_text.len() {
            let end_offset: usize =
                if kind == SyntaxKind::StringLiteral
                    || kind == SyntaxKind::TemplateTail
                    || kind == SyntaxKind::NoSubstitutionTemplateLiteral
                {
                    1
                } else {
                    0
                };
            let end = end - end_offset;
            let start = end - length;
            if &self.file_text[start..end] == text {
                self.offsets.push(start as u32);
                self.offsets.push(end as u32);
                return index;
            }
        }
        let offset = self.file_text.len() + self.other_strings.len();
        self.other_strings.push_str(text);
        self.offsets.push(offset as u32);
        self.offsets.push((offset + length) as u32);
        index
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.encoded_length());
        for &offset in &self.offsets {
            result.extend_from_slice(&offset.to_le_bytes());
        }
        result.extend_from_slice(self.file_text.as_bytes());
        result.extend_from_slice(self.other_strings.as_bytes());
        result
    }

    pub fn string_length(&self) -> usize {
        self.file_text.len() + self.other_strings.len()
    }

    pub fn encoded_length(&self) -> usize {
        self.offsets.len() * 4 + self.file_text.len() + self.other_strings.len()
    }
}

pub fn string_table_add(t: &mut StringTable, text: &str, kind: SyntaxKind, pos: usize, end: usize) -> u32 {
    t.add(text, kind, pos, end)
}

pub fn string_table_encode(t: &StringTable) -> Vec<u8> {
    t.encode()
}

pub fn string_table_encoded_length(t: &StringTable) -> usize {
    t.encoded_length()
}

pub fn string_table_string_length(t: &StringTable) -> usize {
    t.string_length()
}
