use tsox_core::core::text::TextPos;

pub fn compute_position_of_line_and_byte_offset(
    line_starts: &[TextPos],
    line: usize,
    byte_offset: usize,
) -> usize {
    if line >= line_starts.len() {
        panic!(
            "Bad line number. Line: {}, lineStarts.length: {}.",
            line,
            line_starts.len()
        );
    }
    line_starts[line] as usize + byte_offset
}

pub fn compute_position_of_line_and_utf16_character(
    line_starts: &[TextPos],
    line: usize,
    character: usize,
    text: &str,
    allow_edits: bool,
) -> usize {
    let mut line = line;
    if line >= line_starts.len() {
        if allow_edits {
            line = line_starts.len().saturating_sub(1);
        } else {
            panic!(
                "Bad line number. Line: {}, lineStarts.length: {}.",
                line,
                line_starts.len()
            );
        }
    }

    let line_start = line_starts[line] as usize;

    if character > 0 {
        let line_end = if line + 1 < line_starts.len() {
            line_starts[line + 1] as usize
        } else {
            text.len()
        };
        let mut utf16_count = 0usize;
        let mut pos = line_start;
        while pos < line_end {
            if utf16_count >= character {
                break;
            }
            let ch = text[pos..].chars().next().unwrap_or('\0');
            utf16_count += ch.len_utf16();
            pos += ch.len_utf8();
        }
        if !allow_edits {
            if pos == line_end && utf16_count < character {
                panic!(
                    "Bad UTF-16 character offset. Line: {}, character: {}.",
                    line, character
                );
            }
            return pos;
        }
        if pos > text.len() {
            return text.len();
        }
        return pos;
    }

    let res = line_start;
    if allow_edits && res > text.len() {
        return text.len();
    }
    res
}

pub fn default_scanner() -> crate::scanner::token_to_string::Scanner {
    let mut scanner = crate::scanner::token_to_string::Scanner::new("");
    scanner.set_skip_trivia(true);
    scanner
}
