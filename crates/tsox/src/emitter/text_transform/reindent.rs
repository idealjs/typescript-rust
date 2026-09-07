use super::super::UNMAPPED;

pub(crate) fn reindent_and_dedup_tracked(folded: &str, src_offsets: &[u32]) -> (String, Vec<u32>) {
    let chars: Vec<char> = folded.chars().collect();
    let n = chars.len();
    let mut out_text = String::with_capacity(folded.len());
    let mut out_offsets: Vec<u32> = Vec::new();
    let mut depth: i32 = 0;
    let had_trailing_newline = n > 0 && chars[n - 1] == '\n';

    let mut i = 0;
    while i < n {
        let line_start = i;
        while i < n && chars[i] != '\n' {
            i += 1;
        }
        let line_end = i;
        let newline_idx = if i < n && chars[i] == '\n' {
            Some(i)
        } else {
            None
        };
        if i < n && chars[i] == '\n' {
            i += 1;
        }

        let mut content_start = line_start;
        while content_start < line_end && chars[content_start].is_whitespace() {
            content_start += 1;
        }
        let mut content_end = line_end;
        while content_end > content_start && chars[content_end - 1].is_whitespace() {
            content_end -= 1;
        }

        if content_start >= content_end {
            continue;
        }

        let starts_with_close = chars[content_start] == '}';
        let indent_depth = (depth - if starts_with_close { 1 } else { 0 }).max(0);
        for _ in 0..indent_depth {
            out_text.push_str("    ");
            for _ in 0..4 {
                out_offsets.push(UNMAPPED);
            }
        }
        for j in content_start..content_end {
            out_text.push(chars[j]);
            out_offsets.push(src_offsets[j]);
        }
        out_text.push('\n');
        if let Some(nl) = newline_idx {
            out_offsets.push(src_offsets[nl]);
        } else {
            out_offsets.push(src_offsets[content_end - 1]);
        }

        let content: String = chars[content_start..content_end].iter().collect();
        depth += brace_delta(&content);
        if depth < 0 {
            depth = 0;
        }
    }

    if !had_trailing_newline && out_text.ends_with('\n') {
        out_text.pop();
        out_offsets.pop();
    }
    (out_text, out_offsets)
}

pub(crate) fn brace_delta(line: &str) -> i32 {
    let chars: Vec<char> = line.chars().collect();
    let n = chars.len();
    let mut delta = 0i32;
    let mut i = 0;
    while i < n {
        let c = chars[i];
        match c {
            '\'' | '"' | '`' => {
                let quote = c;
                i += 1;
                while i < n {
                    if chars[i] == '\\' {
                        i += 2;
                        continue;
                    }
                    if chars[i] == quote {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
            }
            '/' if i + 1 < n && chars[i + 1] == '/' => {
                break;
            }
            '/' if i + 1 < n && chars[i + 1] == '*' => {
                i += 2;
                while i < n {
                    if chars[i] == '*' && i + 1 < n && chars[i + 1] == '/' {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
            }
            '{' => {
                delta += 1;
                i += 1;
            }
            '}' => {
                delta -= 1;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    delta
}

pub(crate) fn reindent_and_dedup(folded: &str) -> String {
    let mut out = String::with_capacity(folded.len());
    let mut depth: i32 = 0;
    let had_trailing_newline = folded.ends_with('\n');

    for raw_line in folded.split('\n') {
        let stripped = raw_line.trim();
        if stripped.is_empty() {
            continue;
        }
        let starts_with_close = stripped.starts_with('}');
        let indent_depth = (depth - if starts_with_close { 1 } else { 0 }).max(0);
        for _ in 0..indent_depth {
            out.push_str("    ");
        }
        out.push_str(stripped);
        out.push('\n');
        depth += brace_delta(stripped);
        if depth < 0 {
            depth = 0;
        }
    }

    if !had_trailing_newline && out.ends_with('\n') {
        out.pop();
    }
    out
}
