use crate::fourslash::session::Session;

pub use crate::fourslash::api_completions::*;

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
    // Go typeText：逐字符插入后光标推进到插入文本之后
    if let Some(c) = s.cursor.as_mut() {
        *c += delta;
    }
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

pub fn list_diagnostics(s: &Session, file: &str) -> Vec<String> {
    diagnostics_of(s, file)
        .into_iter()
        .map(|d| {
            format!(
                "[{:?}] {} @ {}:{}:{}: {}",
                d.severity,
                d.code.unwrap_or_default(),
                file,
                d.range.start.line,
                d.range.start.character,
                d.message
            )
        })
        .collect()
}

pub fn verify_number_of_errors_in_current_file(s: &Session, expected: usize) {
    let n = diagnostics_of(s, &s.active_file)
        .iter()
        .filter(|d| d.severity.unwrap_or(0) == 1)
        .count();
    assert_eq!(n, expected, "{} 错误数不符", s.active_file);
}

// ---- B4 补全校验（见 api_completions.rs） ----
pub fn verify_quick_info_at(s: &Session, marker: &str, expected: &str, expected_doc: &str) {    let m = s.marker(marker).clone();
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

pub(super) fn line_and_character(text: &str, offset: usize) -> (u32, u32) {
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

pub(super) fn uri_of(_s: &Session, file: &str) -> crate::lsp::lsproto_lsp_uri::DocumentUri {
    let path = super::session::project_path(file);
    crate::lsp::lsproto_lsp_uri::DocumentUri(format!("file://{path}"))
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

/// Go f.FormatDocument：请求格式化并把 TextEdits 应用到文件内容
/// （TextEdits 按 range 从后往前应用，避免偏移漂移）
pub fn format_document(s: &mut Session, filename: &str) {
    let file = if filename.is_empty() { s.active_file.clone() } else { filename.to_string() };
    let uri = uri_of(s, &file);
    let service = s.service.as_ref().expect("无 LanguageService");
    let options = crate::lsp::lsproto_lsp::FormattingOptions::default();
    let edits = service.provide_format_document(&uri, &options);
    apply_edits(s, &file, &edits);
}

/// Go f.FormatSelection：对选区（起止 marker 名）做格式化
pub fn format_selection(s: &mut Session, start_marker: &str, end_marker: &str) {
    let file = s.active_file.clone();
    let start = s.marker(start_marker).position;
    let end = s.marker(end_marker).position;
    let uri = uri_of(s, &file);
    let service = s.service.as_ref().expect("无 LanguageService");
    let options = crate::lsp::lsproto_lsp::FormattingOptions::default();
    let (line0, col0) = line_and_character(&s.file_content(&file), start);
    let (line1, col1) = line_and_character(&s.file_content(&file), end);
    let range = crate::lsp::lsproto_lsp::Range {
        start: crate::lsp::lsproto_lsp::Position { line: line0, character: col0 },
        end: crate::lsp::lsproto_lsp::Position { line: line1, character: col1 },
    };
    let edits = service.provide_format_document_range(&uri, &options, range);
    apply_edits(s, &file, &edits);
}

fn apply_edits(s: &mut Session, file: &str, edits: &[crate::lsp::lsproto_lsp::TextEdit]) {
    if edits.is_empty() {
        return;
    }
    let content = s.file_content(file).to_string();
    // LSP edits 通常已按位置排序；从后往前应用避免偏移漂移
    let mut ranges: Vec<(usize, usize, &str)> = edits
        .iter()
        .map(|e| {
            let content_len = content.len();
            let start = line_col_to_offset(&content, e.range.start.line as usize, e.range.start.character as usize);
            let end = line_col_to_offset(&content, e.range.end.line as usize, e.range.end.character as usize);
            (start.min(content_len), end.min(content_len), e.new_text.as_str())
        })
        .collect();
    ranges.sort_by_key(|(start, end, _)| (*end, std::cmp::Reverse(*start)));
    let mut next = content.clone();
    for (start, end, text) in ranges.iter().rev() {
        let _ = text.len();
        next.replace_range(start..end, text);
    }
    s.set_file_content(file, next);
}

fn line_col_to_offset(text: &str, line: usize, character: usize) -> usize {
    let mut offset = 0usize;
    let mut cur_line = 0usize;
    for (i, b) in text.as_bytes().iter().enumerate() {
        if cur_line == line {
            offset = i;
            // character 按 UTF-16 码元计；ASCII 场景逐字节即可
            return (offset + character).min(text.len());
        }
        if *b == b'\n' {
            cur_line += 1;
        }
    }
    text.len()
}
