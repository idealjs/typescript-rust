#[derive(Debug, Default)]
pub struct LineMap {
    pub line_starts: Vec<u32>,
}

fn is_line_break(ch: char) -> bool {
    matches!(ch, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}

pub fn utf16_len(s: &str) -> usize {
    let mut n = 0usize;
    for c in s.chars() {
        n += c.len_utf16();
    }
    n
}

impl LineMap {
    pub fn from_text(text: &str) -> Self {
        let mut line_starts = Vec::with_capacity(text.matches('\n').count() + 1);
        line_starts.push(0u32);

        let bytes = text.as_bytes();
        let text_len = bytes.len();
        let mut pos = 0usize;

        while pos < text_len {
            let b = bytes[pos];
            if b < 0x80 {
                pos += 1;
                if b == b'\r' {
                    if pos < text_len && bytes[pos] == b'\n' {
                        pos += 1;
                    }
                    line_starts.push(pos as u32);
                } else if b == b'\n' {
                    line_starts.push(pos as u32);
                }
            } else {
                let s = &text[pos..];
                match s.chars().next() {
                    Some(ch) => {
                        pos += ch.len_utf8();
                        if is_line_break(ch) {
                            line_starts.push(pos as u32);
                        }
                    }
                    None => break,
                }
            }
        }

        Self { line_starts }
    }

    pub fn line_at(&self, offset: usize) -> usize {
        match self.line_starts.binary_search(&(offset as u32)) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        }
    }

    pub fn line_start(&self, offset: usize) -> usize {
        let line = self.line_at(offset);
        self.line_starts[line] as usize
    }

    pub fn utf16_column_at(&self, text: &str, offset: usize) -> usize {
        let line_start = self.line_start(offset);
        utf16_len(&text[line_start..offset])
    }
}
