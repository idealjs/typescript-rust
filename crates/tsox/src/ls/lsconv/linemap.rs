pub type LspLineStarts = Vec<usize>;

pub struct LspLineMap {
    pub line_starts: LspLineStarts,
    pub ascii_only: bool,
}

pub fn compute_lsp_line_starts(text: &str) -> LspLineMap {
    let mut line_starts = Vec::with_capacity(text.matches('\n').count() + 1);
    let mut ascii_only = true;

    let text_bytes = text.as_bytes();
    let mut pos = 0usize;
    let mut line_start = 0usize;

    while pos < text_bytes.len() {
        let b = text_bytes[pos];
        if b < 0x80 {
            pos += 1;
            match b {
                b'\r' => {
                    if pos < text_bytes.len() && text_bytes[pos] == b'\n' {
                        pos += 1;
                    }
                    line_starts.push(line_start);
                    line_start = pos;
                }
                b'\n' => {
                    line_starts.push(line_start);
                    line_start = pos;
                }
                _ => {}
            }
        } else {
            let char_len = utf8_char_len(b);
            pos += char_len;
            ascii_only = false;
        }
    }
    line_starts.push(line_start);

    LspLineMap {
        line_starts,
        ascii_only,
    }
}

impl LspLineMap {
    pub fn compute_index_of_line_start(&self, target_pos: usize) -> usize {
        match self.line_starts.binary_search(&target_pos) {
            Ok(idx) => idx,
            Err(idx) => {
                if idx > 0 {
                    idx - 1
                } else {
                    0
                }
            }
        }
    }
}

fn utf8_char_len(first_byte: u8) -> usize {
    if first_byte < 0x80 {
        1
    } else if first_byte < 0xC0 {
        1
    } else if first_byte < 0xE0 {
        2
    } else if first_byte < 0xF0 {
        3
    } else {
        4
    }
}
