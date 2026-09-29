#![allow(dead_code, unused_imports, unused_variables)]

use super::m5u_3::{is_only_spaces_or_tabs, scan_non_whitespace, starts_with_single_line_whitespace};

pub fn strip_js_doc_template_indentation(template: &str, new_line: &str) -> String {
    let mut lines: Vec<String> = template.split(new_line).map(|l| l.to_string()).collect();
    for line in lines.iter_mut() {
        let trimmed = line.trim_start_matches([' ', '\t']).to_string();
        if trimmed.starts_with('/') {
            *line = trimmed;
        } else if trimmed.starts_with('*') {
            *line = format!(" {}", trimmed);
        }
    }
    lines.join(new_line)
}

fn line_has_only_js_doc_asterisk(line: &str) -> bool {
    let trimmed = line.trim_matches([' ', '\t']);
    !trimmed.is_empty() && trimmed.chars().all(|c| c == '*')
}

pub fn transform_js_doc_template_lines(template: &str, new_line: &str, snippet_index: &mut i32) -> String {
    let mut lines: Vec<String> = template.split(new_line).map(|l| l.to_string()).collect();
    for i in 0..lines.len() {
        let line = lines[i].clone();
        if i > 0 && lines[i - 1].starts_with("/**") && line_has_only_js_doc_asterisk(&line) {
            lines[i] = format!("{}$0", line);
            continue;
        }
        if let Some(transformed) = transform_js_doc_param_line(&line, snippet_index) {
            lines[i] = transformed;
            continue;
        }
        if let Some(transformed) = transform_js_doc_returns_line(&line, snippet_index) {
            lines[i] = transformed;
        }
    }
    lines.join(new_line)
}

pub fn transform_js_doc_param_line(line: &str, snippet_index: &mut i32) -> Option<String> {
    let mut prefix = "";
    let mut rest = line;
    if rest.starts_with(' ') {
        prefix = " ";
        rest = &rest[1..];
    }
    if !rest.starts_with("* @param") {
        return None;
    }
    rest = &rest["* @param".len()..];
    if !starts_with_single_line_whitespace(rest) {
        return None;
    }
    rest = rest.trim_start_matches([' ', '\t']);

    let mut type_text = String::new();
    if rest.starts_with('{') {
        let close_brace = rest.find('}')?;
        type_text = format!(" {}", &rest[..close_brace + 1]);
        rest = &rest[close_brace + 1..];
        if !starts_with_single_line_whitespace(rest) {
            return None;
        }
        rest = rest.trim_start_matches([' ', '\t']);
    }

    let (param_name, rest2) = scan_non_whitespace(rest)?;
    if !is_only_spaces_or_tabs(rest2) {
        return None;
    }

    let mut out = format!("{}* @param ", prefix);
    if type_text == " {any}" || type_text == " {*}" {
        out.push_str("{${");
        out.push_str(&snippet_index.to_string());
        out.push_str(":*} ");
        *snippet_index += 1;
    } else if !type_text.is_empty() {
        out.push_str(&type_text);
        out.push(' ');
    }
    out.push_str(&format!("{} ${{{}}}", param_name, snippet_index));
    *snippet_index += 1;
    Some(out)
}

pub fn transform_js_doc_returns_line(line: &str, snippet_index: &mut i32) -> Option<String> {
    let mut prefix = "";
    let mut rest = line;
    if rest.starts_with(' ') {
        prefix = " ";
        rest = &rest[1..];
    }
    if !rest.starts_with("* @returns") || !is_only_spaces_or_tabs(&rest["* @returns".len()..]) {
        return None;
    }
    let text = format!("{}* @returns ${{{}}}", prefix, snippet_index);
    *snippet_index += 1;
    Some(text)
}
