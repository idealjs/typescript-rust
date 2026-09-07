#![allow(unused_imports)]

use super::*;

pub(crate) fn iterate_comment_ranges(text: &str, pos: usize, trailing: bool) -> Vec<CommentRange> {
    let bytes = text.as_bytes();
    let text_len = bytes.len();
    let mut pos = pos;
    let mut result: Vec<CommentRange> = Vec::new();

    let mut pending_pos: usize = 0;
    let mut pending_end: usize = 0;
    let mut pending_kind: CommentRangeKind = CommentRangeKind::SingleLine;
    let mut pending_has_trailing_new_line = false;
    let mut has_pending = false;

    let mut collecting = trailing;
    if pos == 0 {
        collecting = true;
        if is_shebang_trivia(text, pos) {
            pos = scan_shebang_trivia(text, pos);
        }
    }

    while pos < text_len {
        let (ch, size) = decode_char(text, pos);
        match ch {
            '\r' => {
                if pos + 1 < text_len && bytes[pos + 1] == b'\n' {
                    pos += 1;
                }
                pos += 1;
                if trailing {
                    break;
                }
                collecting = true;
                if has_pending {
                    pending_has_trailing_new_line = true;
                }
                continue;
            }
            '\n' => {
                pos += 1;
                if trailing {
                    break;
                }
                collecting = true;
                if has_pending {
                    pending_has_trailing_new_line = true;
                }
                continue;
            }
            '\t' | '\x0B' | '\x0C' | ' ' => {
                pos += 1;
                continue;
            }
            '/' => {
                let mut next_char = b'\0';
                if pos + 1 < text_len {
                    next_char = bytes[pos + 1];
                }
                let mut has_trailing_new_line = false;
                if next_char == b'/' || next_char == b'*' {
                    let kind = if next_char == b'/' {
                        CommentRangeKind::SingleLine
                    } else {
                        CommentRangeKind::MultiLine
                    };
                    let start_pos = pos;
                    pos += 2;
                    if next_char == b'/' {
                        while pos < text_len {
                            let (c, s) = decode_char(text, pos);
                            if is_line_break(c) {
                                has_trailing_new_line = true;
                                break;
                            }
                            pos += s;
                        }
                    } else {
                        if let Some(i) = text[pos..].find("*/") {
                            pos += i + 2;
                        } else {
                            pos = text_len;
                        }
                    }
                    if collecting {
                        if has_pending {
                            result.push(CommentRange {
                                pos: pending_pos,
                                end: pending_end,
                                kind: pending_kind,
                                has_trailing_new_line: pending_has_trailing_new_line,
                            });
                        }
                        pending_pos = start_pos;
                        pending_end = pos;
                        pending_kind = kind;
                        pending_has_trailing_new_line = has_trailing_new_line;
                        has_pending = true;
                    }
                    continue;
                }
                break;
            }
            _ => {
                if ch > '\u{7F}' && is_whitespace_like(ch) {
                    if has_pending && is_line_break(ch) {
                        pending_has_trailing_new_line = true;
                    }
                    pos += size;
                    continue;
                }
                break;
            }
        }
    }
    if has_pending {
        result.push(CommentRange {
            pos: pending_pos,
            end: pending_end,
            kind: pending_kind,
            has_trailing_new_line: pending_has_trailing_new_line,
        });
    }
    result
}
