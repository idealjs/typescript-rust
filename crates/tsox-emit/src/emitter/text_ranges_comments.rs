pub(crate) fn collect_all_comment_ranges(text: &str) -> Vec<(usize, usize)> {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    let mut pos = 0usize;

    let mut prev_significant: char = ';';

    while pos < len {
        let b = bytes[pos];
        match b {
            b'/' if pos + 1 < len && bytes[pos + 1] == b'/' => {
                let start = pos;
                pos += 2;
                while pos < len && bytes[pos] != b'\n' && bytes[pos] != b'\r' {
                    pos += 1;
                }
                ranges.push((start, pos));
            }
            b'/' if pos + 1 < len && bytes[pos + 1] == b'*' => {
                let start = pos;
                pos += 2;
                while pos < len {
                    if bytes[pos] == b'*' && pos + 1 < len && bytes[pos + 1] == b'/' {
                        pos += 2;
                        break;
                    }
                    pos += 1;
                }
                ranges.push((start, pos));
            }
            b'/' => {
                if is_regex_context(prev_significant) {
                    let start = pos;
                    pos += 1;
                    let mut in_class = false;
                    while pos < len {
                        let c = bytes[pos];
                        if c == b'\\' && pos + 1 < len {
                            pos += 2;
                            continue;
                        }
                        if c == b'[' {
                            in_class = true;
                        }
                        if c == b']' {
                            in_class = false;
                        }
                        if c == b'/' && !in_class {
                            pos += 1;

                            while pos < len && is_regex_flag_char(bytes[pos]) {
                                pos += 1;
                            }
                            break;
                        }
                        if c == b'\n' {
                            break;
                        }
                        pos += 1;
                    }
                    let _ = start;
                } else {
                    pos += 1;
                }
                prev_significant = '/';
            }
            b'\'' | b'"' => {
                let quote = b;
                pos += 1;
                while pos < len {
                    let c = bytes[pos];
                    if c == b'\\' && pos + 1 < len {
                        pos += 2;
                        continue;
                    }
                    if c == quote {
                        pos += 1;
                        break;
                    }
                    if c == b'\n' {
                        break;
                    }
                    pos += 1;
                }
                prev_significant = char::from(quote);
            }
            b'`' => {
                prev_significant = '`';
                pos += 1;
                skip_template_literal(text, &mut pos);
            }
            b' ' | b'\t' | b'\n' | b'\r' => {
                pos += 1;
            }
            _ => {
                prev_significant = char::from(b);
                pos += 1;
            }
        }
    }

    ranges
}

pub(crate) fn skip_template_literal(text: &str, pos: &mut usize) {
    let bytes = text.as_bytes();
    let len = bytes.len();
    while *pos < len {
        let b = bytes[*pos];
        if b == b'\\' && *pos + 1 < len {
            *pos += 2;
            continue;
        }
        if b == b'`' {
            *pos += 1;
            return;
        }
        if b == b'$' && *pos + 1 < len && bytes[*pos + 1] == b'{' {
            *pos += 2;
            let mut depth = 1;
            while *pos < len && depth > 0 {
                let c = bytes[*pos];
                match c {
                    b'{' => {
                        depth += 1;
                        *pos += 1;
                    }
                    b'}' => {
                        depth -= 1;
                        *pos += 1;
                    }
                    b'\'' | b'"' => {
                        let quote = c;
                        *pos += 1;
                        while *pos < len {
                            if bytes[*pos] == b'\\' && *pos + 1 < len {
                                *pos += 2;
                                continue;
                            }
                            if bytes[*pos] == quote {
                                *pos += 1;
                                break;
                            }
                            *pos += 1;
                        }
                    }
                    b'`' => {
                        *pos += 1;
                        skip_template_literal(text, pos);
                    }
                    _ => {
                        *pos += 1;
                    }
                }
            }
        } else {
            *pos += 1;
        }
    }
}

pub(crate) fn is_regex_context(prev: char) -> bool {
    matches!(
        prev,
        '(' | ','
            | '='
            | ':'
            | '['
            | '!'
            | '&'
            | '|'
            | '?'
            | '{'
            | '}'
            | ';'
            | '<'
            | '>'
            | '+'
            | '-'
            | '*'
            | '/'
            | '%'
            | '~'
            | '^'
            | '\n'
            | '\0'
    )
}

pub(crate) fn is_regex_flag_char(b: u8) -> bool {
    matches!(b, b'g' | b'i' | b'm' | b's' | b'u' | b'y' | b'd' | b'v')
}
