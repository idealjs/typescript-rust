#![allow(dead_code, unused_imports, unused_variables)]

pub fn lower_first_char(s: &str) -> String { crate::fntrace::enter("lower_first_char"); 
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_lowercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

pub(crate) fn match_slash_replacer(s: &str) -> &str { crate::fntrace::enter("match_slash_replacer"); 
    &s[1..]
}
