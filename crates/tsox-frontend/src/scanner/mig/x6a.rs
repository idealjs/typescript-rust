#![allow(unused_imports)]
#![allow(dead_code)]

pub fn has_js_doc_tag(text: &str, tags: &[&str]) -> bool {
    for tag in tags {
        let Some(rest) = text.strip_prefix(tag) else {
            continue;
        };
        if rest.is_empty() {
            return true;
        }
        let ch = rest.chars().next().unwrap();
        if ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' || ch == '}' || ch == '*' {
            return true;
        }
    }
    false
}
