#![allow(dead_code, unused_imports, unused_variables)]

pub fn cut_any(s: &str, cutset: &str) -> (String, String, bool) { crate::fntrace::enter("cut_any"); 
    if let Some(i) = s.find(|c: char| cutset.contains(c)) {
        let before = &s[..i];
        let after_and_found = &s[i..];
        let c = after_and_found.chars().next().unwrap();
        let after = &after_and_found[c.len_utf8()..];
        (
            before.to_string(),
            after.to_string(),
            true,
        )
    } else {
        (s.to_string(), String::new(), false)
    }
}
