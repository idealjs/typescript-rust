use crate::emitter::commonjs::*;

pub(crate) fn rewrite_import_extensions_tracked(
    text: &str,
    src_offsets: &[u32],
) -> (String, Vec<u32>) {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut out_text = String::with_capacity(text.len());
    let mut out_offsets = Vec::with_capacity(n);
    let mut i = 0;
    while i < n {
        if i + 5 <= n
            && chars[i] == 'f'
            && chars[i + 1] == 'r'
            && chars[i + 2] == 'o'
            && chars[i + 3] == 'm'
            && chars[i + 4] == ' '
        {
            i = copy_string_literal_tracked(
                &chars,
                src_offsets,
                i,
                &mut out_text,
                &mut out_offsets,
            );
        } else if i + 7 <= n
            && chars[i] == 'i'
            && chars[i + 1] == 'm'
            && chars[i + 2] == 'p'
            && chars[i + 3] == 'o'
            && chars[i + 4] == 'r'
            && chars[i + 5] == 't'
            && chars[i + 6] == '('
        {
            for j in 0..7 {
                out_text.push(chars[i + j]);
                out_offsets.push(src_offsets[i + j]);
            }
            i += 7;
            while i < n && chars[i].is_ascii_whitespace() {
                out_text.push(chars[i]);
                out_offsets.push(src_offsets[i]);
                i += 1;
            }
            if i < n && (chars[i] == '"' || chars[i] == '\'') {
                i = copy_string_literal_tracked(
                    &chars,
                    src_offsets,
                    i,
                    &mut out_text,
                    &mut out_offsets,
                );
            }
        } else {
            out_text.push(chars[i]);
            out_offsets.push(src_offsets[i]);
            i += 1;
        }
    }
    (out_text, out_offsets)
}

pub(crate) fn copy_string_literal_tracked(
    chars: &[char],
    src_offsets: &[u32],
    start: usize,
    out_text: &mut String,
    out_offsets: &mut Vec<u32>,
) -> usize {
    let mut i = start;
    if chars[i] == 'f' {
        for _ in 0..5 {
            out_text.push(chars[i]);
            out_offsets.push(src_offsets[i]);
            i += 1;
        }
        while i < chars.len() && chars[i].is_ascii_whitespace() {
            out_text.push(chars[i]);
            out_offsets.push(src_offsets[i]);
            i += 1;
        }
    }

    if i < chars.len() && (chars[i] == '"' || chars[i] == '\'') {
        let quote = chars[i];
        out_text.push(quote);
        out_offsets.push(src_offsets[i]);
        i += 1;
        let spec_start = i;
        while i < chars.len() && chars[i] != quote {
            i += 1;
        }
        let specifier: String = chars[spec_start..i].iter().collect();
        let rewritten = rewrite_one_specifier(&specifier);
        let spec_len = i - spec_start;
        for (j, rc) in rewritten.chars().enumerate() {
            out_text.push(rc);
            if j < spec_len {
                out_offsets.push(src_offsets[spec_start + j]);
            } else {
                out_offsets.push(src_offsets[spec_start + spec_len - 1]);
            }
        }
        if i < chars.len() {
            out_text.push(chars[i]);
            out_offsets.push(src_offsets[i]);
            i += 1;
        }
    }
    i
}
