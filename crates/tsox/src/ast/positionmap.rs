#[derive(Debug)]
pub struct PositionMap {
    ascii_only: bool,
    entries: Vec<PositionMapEntry>,
}

#[derive(Debug)]
struct PositionMapEntry {
    utf8_pos: usize,

    delta: usize,
}

pub fn compute_position_map(text: &str) -> PositionMap {
    let mut entries = Vec::new();
    let mut delta = 0usize;

    for (byte_offset, ch) in text.char_indices() {
        let utf8_size = ch.len_utf8();
        if utf8_size <= 1 {
            continue;
        }
        let utf16_size = ch.len_utf16();
        delta += utf8_size - utf16_size;
        entries.push(PositionMapEntry {
            utf8_pos: byte_offset + utf8_size,
            delta,
        });
    }

    let ascii_only = entries.is_empty();
    PositionMap {
        ascii_only,
        entries,
    }
}

impl PositionMap {
    pub fn is_ascii_only(&self) -> bool {
        self.ascii_only
    }

    pub fn utf8_to_utf16(&self, utf8_offset: usize) -> usize {
        if self.ascii_only {
            return utf8_offset;
        }

        let lo = self.entries.partition_point(|e| e.utf8_pos <= utf8_offset);
        if lo == 0 {
            return utf8_offset;
        }
        utf8_offset - self.entries[lo - 1].delta
    }

    pub fn utf16_to_utf8(&self, utf16_offset: usize) -> usize {
        if self.ascii_only {
            return utf16_offset;
        }

        let lo = self
            .entries
            .partition_point(|e| e.utf8_pos - e.delta <= utf16_offset);
        if lo == 0 {
            return utf16_offset;
        }
        utf16_offset + self.entries[lo - 1].delta
    }
}

#[cfg(test)]
mod tests;
