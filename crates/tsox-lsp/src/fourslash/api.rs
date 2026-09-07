//! fourslash 框架操作：自由函数 + 显式 Session 参数。
//! v1 覆盖会话状态类操作；LSP 查询/断言族随用例批次补齐。

use crate::fourslash::session::Session;

pub fn go_to_marker(s: &mut Session, name: &str) {
    let m = s.marker(name).clone();
    s.active_file = m.file_name.clone();
    s.cursor = Some(m.position);
}

pub fn go_to_file(s: &mut Session, name: &str) {
    s.file_content(name); // 校验存在
    s.active_file = name.to_string();
}

pub fn insert(s: &mut Session, text: &str) {
    let pos = s
        .cursor
        .unwrap_or_else(|| panic!("insert 前无光标（go_to_marker）"));
    let file = s.active_file.clone();
    let content = s.file_content(&file).to_string();
    let char_pos = byte_index_of_char(&content, pos);
    let mut next = String::with_capacity(content.len() + text.len());
    next.push_str(&content[..char_pos]);
    next.push_str(text);
    next.push_str(&content[char_pos..]);
    s.set_file_content(&file, next);
    // 编辑后平移同文件 marker/range（对齐 Go fourslash editScriptAndUpdateMarkers）
    let delta: usize = text.chars().count();
    for m in &mut s.data.markers {
        if m.file_name == file && m.position > pos {
            m.position += delta;
        }
    }
    for r in &mut s.data.ranges {
        if r.file_name == file {
            if r.start > pos {
                r.start += delta;
            }
            if r.end > pos {
                r.end += delta;
            }
        }
    }
    s.data.marker_positions = s
        .data
        .markers
        .iter()
        .filter_map(|m| m.name.clone().map(|n| (n, m.position)))
        .collect();
}

pub fn verify_current_line_content(s: &Session, expected: &str) {
    let content = s.file_content(&s.active_file);
    let line = current_line(content, s.cursor);
    assert_eq!(line, expected, "行内容不符（{}）", s.active_file);
}

pub fn verify_current_file_content(s: &Session, expected: &str) {
    let content = s.file_content(&s.active_file);
    assert_eq!(content, expected, "文件内容不符（{}）", s.active_file);
}

/// 未实现的框架操作（生成器标记对应用例 #[ignore]）
pub fn unsupported(method: &str) {
    panic!("fourslash 方法未实现: {method}");
}

fn byte_index_of_char(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(b, _)| b)
        .unwrap_or(text.len())
}

fn current_line(text: &str, cursor: Option<usize>) -> &str {
    let pos = cursor.unwrap_or(0);
    let byte = byte_index_of_char(text, pos);
    let start = text[..byte].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let end = text[byte..]
        .find('\n')
        .map(|i| byte + i)
        .unwrap_or(text.len());
    &text[start..end]
}

// ---- B1 会话查询 ----

pub fn ranges(s: &Session) -> Vec<(String, usize, usize)> {
    s.data
        .ranges
        .iter()
        .map(|r| (r.file_name.clone(), r.start, r.end))
        .collect()
}

pub fn markers(s: &Session) -> Vec<(String, usize)> {
    s.data
        .markers
        .iter()
        .map(|m| (m.file_name.clone(), m.position))
        .collect()
}

// ---- B2 诊断 ----

fn diagnostics_of(s: &Session, file: &str) -> Vec<crate::ls::types::Diagnostic> {
    let service = s.service.as_ref().expect("无 LanguageService");
    service.provide_diagnostics(&uri_of(s, file))
}

pub fn verify_no_errors(s: &Session) {
    for f in &s.data.files {
        for d in diagnostics_of(s, &f.file_name) {
            assert!(
                d.severity.unwrap_or(0) != 1,
                "{} 出现错误诊断: {}",
                f.file_name,
                d.message
            );
        }
    }
}

pub fn verify_number_of_errors_in_current_file(s: &Session, expected: usize) {
    let n = diagnostics_of(s, &s.active_file)
        .iter()
        .filter(|d| d.severity.unwrap_or(0) == 1)
        .count();
    assert_eq!(n, expected, "{} 错误数不符", s.active_file);
}

// ---- B3 quickinfo ----

pub fn verify_quick_info_at(s: &Session, marker: &str, expected: &str, expected_doc: &str) {
    let m = s.marker(marker).clone();
    let content = s.file_content(&m.file_name).to_string();
    let (line, character) = line_and_character(&content, m.position);
    let service = s.service.as_ref().expect("无 LanguageService");
    let hover = service.provide_hover(
        &uri_of(s, &m.file_name),
        crate::lsp::lsproto_lsp_basic::Position { line, character },
    );
    let Some(hover) = hover else {
        panic!("{}:{} 无 quick info", m.file_name, marker);
    };
    let text = hover
        .contents
        .string
        .clone()
        .or_else(|| {
            hover
                .contents
                .markup_content
                .as_ref()
                .map(|m| m.value.clone())
        })
        .unwrap_or_default();
    let bare = strip_code_fence(&text);
    let (main, rest) = match bare.split_once("\n\n") {
        Some((m, r)) => (m, r),
        None => (bare.as_str(), ""),
    };
    assert_eq!(main.trim(), expected.trim(), "quick info 不符（{marker}）");
    if !expected_doc.is_empty() {
        assert!(
            rest.contains(expected_doc.trim()) || text.contains(expected_doc.trim()),
            "quick info 缺少文档: {text}"
        );
    }
}

fn line_and_character(text: &str, offset: usize) -> (u32, u32) {
    let chars: Vec<char> = text.chars().collect();
    let off = offset.min(chars.len());
    let mut line = 0u32;
    let mut start = 0usize;
    for (i, c) in chars.iter().enumerate().take(off) {
        if *c == '\n' {
            line += 1;
            start = i + 1;
        }
    }
    (line, (off - start) as u32)
}

fn uri_of(_s: &Session, file: &str) -> crate::lsp::lsproto_lsp_uri::DocumentUri {
    if file.starts_with('/') {
        return crate::lsp::lsproto_lsp_uri::DocumentUri(format!("file://{file}"));
    }
    crate::lsp::lsproto_lsp_uri::DocumentUri(format!(
        "file://{}/{}",
        super::session::PROJECT_ROOT,
        file
    ))
}

fn strip_code_fence(text: &str) -> String {
    let t = text.trim();
    if let Some(rest) = t.strip_prefix("```") {
        if let Some(end) = rest.rfind("```") {
            let body = &rest[..end];
            let body = body.split_once('\n').map(|(_, b)| b).unwrap_or(body);
            return body.trim().to_string();
        }
    }
    t.to_string()
}
