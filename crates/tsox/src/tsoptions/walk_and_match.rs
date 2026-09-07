#![allow(unused_imports)]

use super::*;

pub(crate) fn walk_and_match(root_spec: &str, dir: &str, fs: &dyn FS, results: &mut Vec<String>) {
    let entries = fs.get_accessible_entries(dir);
    for file in &entries.files {
        let full = tspath::combine_paths(dir, &[file]);
        if glob_matches(root_spec, &full) {
            results.push(full);
        }
    }
    for d in &entries.directories {
        if d.eq_ignore_ascii_case("node_modules")
            || d.eq_ignore_ascii_case("bower_components")
            || d.eq_ignore_ascii_case("jspm_packages")
            || d == ".git"
        {
            continue;
        }
        let full = tspath::combine_paths(dir, &[d]);
        walk_and_match(root_spec, &full, fs, results);
    }
}

pub(crate) fn glob_matches(spec: &str, path: &str) -> bool {
    match Glob::parse(spec) {
        Ok(g) => g.is_match(path),
        Err(_) => false,
    }
}

pub(crate) fn strip_jsonc(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0usize;
    let mut in_string = false;
    while i < chars.len() {
        let c = chars[i];
        if in_string {
            out.push(c);
            if c == '\\' && i + 1 < chars.len() {
                out.push(chars[i + 1]);
                i += 2;
                continue;
            }
            if c == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        match c {
            '"' => {
                in_string = true;
                out.push(c);
                i += 1;
            }
            '/' if i + 1 < chars.len() && chars[i + 1] == '/' => {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            '/' if i + 1 < chars.len() && chars[i + 1] == '*' => {
                i += 2;
                while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                    i += 1;
                }
                i += 2;
            }
            ',' if i + 1 < chars.len() => {
                let mut j = i + 1;
                while j < chars.len() && chars[j].is_whitespace() {
                    j += 1;
                }
                if j < chars.len() && (chars[j] == '}' || chars[j] == ']') {
                    i += 1;
                } else {
                    out.push(c);
                    i += 1;
                }
            }
            _ => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}
