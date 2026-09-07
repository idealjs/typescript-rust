#![allow(unused_imports)]

use super::*;

pub(crate) fn is_conflict_marker_trivia(text: &str, pos: usize) -> bool {
    let bytes = text.as_bytes();
    let text_len = bytes.len();
    if pos + 1 >= text_len || bytes[pos + 1] != bytes[pos] {
        return false;
    }

    let mut at_line_start = pos == 0 || is_line_break(bytes[pos - 1] as char);
    if !at_line_start && pos >= 2 {
        at_line_start = is_line_break(bytes[pos - 2] as char);
    }
    if at_line_start && pos + MERGE_CONFLICT_MARKER_LENGTH < text_len {
        let ch = bytes[pos];
        for i in 0..MERGE_CONFLICT_MARKER_LENGTH {
            if bytes[pos + i] != ch {
                return false;
            }
        }

        return ch == b'=' || bytes[pos + MERGE_CONFLICT_MARKER_LENGTH] == b' ';
    }
    false
}

pub(crate) fn scan_conflict_marker_trivia(
    text: &str,
    pos: usize,
    report_error: Option<&dyn Fn(usize, usize)>,
) -> usize {
    if let Some(report) = report_error {
        report(pos, MERGE_CONFLICT_MARKER_LENGTH);
    }
    let bytes = text.as_bytes();
    let text_len = bytes.len();
    let (ch, _size) = decode_char(text, pos);
    let mut pos = pos;
    if ch == '<' || ch == '>' {
        while pos < text_len && !is_line_break(bytes[pos] as char) {
            pos += 1;
        }
    } else {
        while pos < text_len {
            let current = bytes[pos];
            if (current == b'=' || current == b'>')
                && current as char != ch
                && is_conflict_marker_trivia(text, pos)
            {
                break;
            }
            pos += 1;
        }
    }
    pos
}

pub fn skip_trivia(text: &str, pos: usize) -> usize {
    skip_trivia_ex(text, pos, &SkipTriviaOptions::default(), None)
}

pub fn skip_trivia_ex(
    text: &str,
    pos: usize,
    options: &SkipTriviaOptions,
    report_error: Option<&dyn Fn(usize, usize)>,
) -> usize {
    let bytes = text.as_bytes();
    let text_len = bytes.len();
    let mut pos = pos;

    let mut can_consume_star = false;
    loop {
        if pos >= text_len {
            return pos;
        }
        let c = bytes[pos] as char;
        match c {
            '\r' => {
                if pos + 1 < text_len && bytes[pos + 1] == b'\n' {
                    pos += 1;
                }
                pos += 1;
                if options.stop_after_line_break {
                    return pos;
                }
                can_consume_star = options.in_jsdoc;
                continue;
            }
            '\n' => {
                pos += 1;
                if options.stop_after_line_break {
                    return pos;
                }
                can_consume_star = options.in_jsdoc;
                continue;
            }
            '\t' | '\x0B' | '\x0C' | ' ' => {
                pos += 1;
                continue;
            }
            '/' => {
                if options.stop_at_comments {
                    return pos;
                }
                if pos + 1 < text_len {
                    if bytes[pos + 1] == b'/' {
                        pos += 2;
                        while pos < text_len {
                            let (ch, size) = decode_char(text, pos);
                            if is_line_break(ch) {
                                break;
                            }
                            pos += size;
                        }
                        can_consume_star = false;
                        continue;
                    }
                    if bytes[pos + 1] == b'*' {
                        pos += 2;
                        while pos < text_len {
                            if bytes[pos] == b'*' && pos + 1 < text_len && bytes[pos + 1] == b'/' {
                                pos += 2;
                                break;
                            }
                            let (_, size) = decode_char(text, pos);
                            pos += size;
                        }
                        can_consume_star = false;
                        continue;
                    }
                }
                return pos;
            }
            '<' | '|' | '=' | '>' => {
                if is_conflict_marker_trivia(text, pos) {
                    pos = scan_conflict_marker_trivia(text, pos, report_error);
                    can_consume_star = false;
                    continue;
                }
                return pos;
            }
            '#' => {
                if pos == 0 && is_shebang_trivia(text, pos) {
                    pos = scan_shebang_trivia(text, pos);
                    continue;
                }
                return pos;
            }
            '*' => {
                if can_consume_star {
                    pos += 1;
                    can_consume_star = false;
                    continue;
                }
                return pos;
            }
            _ => {
                let (ch, size) = decode_char(text, pos);
                if ch > '\u{7F}' && is_whitespace_like(ch) {
                    pos += size;
                    continue;
                }
                return pos;
            }
        }
    }
}

pub fn get_leading_comment_ranges(text: &str, pos: usize) -> Vec<CommentRange> {
    iterate_comment_ranges(text, pos, false)
}

pub fn get_trailing_comment_ranges(text: &str, pos: usize) -> Vec<CommentRange> {
    iterate_comment_ranges(text, pos, true)
}
