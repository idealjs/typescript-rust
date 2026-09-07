#![allow(unused_imports)]

use super::*;

pub(crate) fn rewrite_import_extensions(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let is_char_start = (bytes[i] & 0xC0) != 0x80;

        if is_char_start && i + 5 <= bytes.len() && &bytes[i..i + 5] == b"from " {
            result.push_str("from ");
            i += 5;

            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                result.push(bytes[i] as char);
                i += 1;
            }

            if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                let quote = bytes[i] as char;
                let start = i + 1;
                result.push(quote);
                i += 1;
                while i < bytes.len() && bytes[i] != quote as u8 {
                    if (bytes[i] & 0x80) == 0 {
                        i += 1;
                    } else if (bytes[i] & 0xE0) == 0xC0 {
                        i += 2;
                    } else if (bytes[i] & 0xF0) == 0xE0 {
                        i += 3;
                    } else {
                        i += 4;
                    }
                }
                let specifier = &text[start..i];
                let rewritten = rewrite_one_specifier(specifier);
                result.push_str(&rewritten);
                if i < bytes.len() {
                    result.push(quote);
                    i += 1;
                }
            }
        } else if is_char_start && i + 7 <= bytes.len() && &bytes[i..i + 7] == b"import(" {
            result.push_str("import(");
            i += 7;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                result.push(bytes[i] as char);
                i += 1;
            }
            if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                let quote = bytes[i] as char;
                let start = i + 1;
                result.push(quote);
                i += 1;
                while i < bytes.len() && bytes[i] != quote as u8 {
                    if (bytes[i] & 0x80) == 0 {
                        i += 1;
                    } else if (bytes[i] & 0xE0) == 0xC0 {
                        i += 2;
                    } else if (bytes[i] & 0xF0) == 0xE0 {
                        i += 3;
                    } else {
                        i += 4;
                    }
                }
                let specifier = &text[start..i];
                let rewritten = rewrite_one_specifier(specifier);
                result.push_str(&rewritten);
                if i < bytes.len() {
                    result.push(quote);
                    i += 1;
                }
            }
        } else {
            let ch = text[i..].chars().next().unwrap();
            result.push(ch);
            i += ch.len_utf8();
        }
    }
    result
}

pub(crate) fn rewrite_one_specifier(spec: &str) -> String {
    for (old, new) in [
        (".ts", ".js"),
        (".tsx", ".js"),
        (".mts", ".mjs"),
        (".cts", ".cjs"),
    ] {
        if spec.ends_with(old) {
            return format!("{}{}", &spec[..spec.len() - old.len()], new);
        }
    }
    spec.to_string()
}
