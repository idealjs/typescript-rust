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

/// Go typeText：逐字符插入；每敲一个字符按 Go 语义发送 OnTypeFormatting
/// 并应用返回的编辑（光标与 marker 随编辑平移）。
pub fn insert(s: &mut Session, text: &str) {
    for ch in text.chars() {
        let pos = s
            .cursor
            .unwrap_or_else(|| panic!("insert 前无光标（go_to_marker）"));
        let ch_text = ch.to_string();
        edit_script(s, pos, pos, &ch_text);
        let mut offset = pos + 1;
        s.cursor = Some(offset);

        if s.enable_formatting {
            let edits = on_type_formatting(s, offset, &ch_text);
            if !edits.is_empty() {
                offset = (offset as isize + apply_text_edits(s, &edits)).max(0) as usize;
                s.cursor = Some(offset);
            }
        }
    }
}

fn apply_text_edits_lsp(s: &mut Session, edits: &[crate::lsp::lsproto_lsp::TextEdit]) {
    if edits.is_empty() {
        return;
    }
    apply_text_edits(s, edits);
}

/// Go GoToBOF
pub fn go_to_bof(s: &mut Session) {
    s.cursor = Some(0);
}

/// Go InsertLine：在光标处输入文本 + 换行（逐字符触发 on-type 格式化）
pub fn insert_line(s: &mut Session, text: &str) {
    let combined = format!("{text}\n");
    insert(s, &combined);
}

/// Go Configure：覆盖 FormatCodeSettings 后重建服务
pub fn configure_format_settings(s: &mut Session, settings: &[(&str, &str)]) {
    {
        let mut prefs = s.prefs.lock().unwrap();
        for (name, value) in settings {
            apply_format_setting(&mut prefs.format_code_settings, name, value);
        }
    }
    let file_names = s.data.files.iter().map(|f| f.file_name.clone()).collect();
    s.rebuild_service(file_names);
}

fn apply_format_setting(settings: &mut crate::ls::lsutil::FormatCodeSettings, name: &str, value: &str) {
    let tristate = match value {
        "true" => tsox_core::core::tristate::Tristate::True,
        "false" => tsox_core::core::tristate::Tristate::False,
        _ => tsox_core::core::tristate::Tristate::Unknown,
    };
    crate::ls::lsutil::set_format_code_setting(settings, name, tristate, value);
}

/// Go Paste：文本以粘贴方式进入，随后对粘贴范围做 range formatting。
pub fn paste(s: &mut Session, text: &str) {
    let pos = s
        .cursor
        .unwrap_or_else(|| panic!("paste 前无光标（go_to_marker）"));
    edit_script(s, pos, pos, text);
    if !s.enable_formatting {
        let delta: usize = text.chars().count();
        s.cursor = Some(pos + delta);
        return;
    }
    let end = pos + text.chars().count();
    let edits = range_formatting(s, pos, end);
    if !edits.is_empty() {
        apply_text_edits(s, &edits);
    }
}

/// Go GoToEOF
pub fn go_to_eof(s: &mut Session) {
    let content = s.file_content(&s.active_file);
    s.cursor = Some(content.chars().count());
}

/// Go SetPreference：按原始键名改写用户偏好
pub fn set_user_preference(s: &mut Session, name: &str, value: &str) {
    {
        let mut prefs = s.prefs.lock().unwrap();
        crate::ls::lsutil_user_preferences::set_user_preference_raw(&mut prefs, name, value);
    }
    let file_names: Vec<String> = s
        .data
        .files
        .iter()
        .map(|f| super::session::project_path(&f.file_name))
        .collect();
    s.rebuild_service(file_names);
}

/// Go GoToPosition：光标移到活动文件绝对偏移处
pub fn go_to_position(s: &mut Session, offset: usize) {
    s.cursor = Some(offset);
}

/// Go DeleteAtCaret：在光标处向后删除 count 个字符（光标不动）
pub fn delete_at_caret(s: &mut Session, count: usize) {
    for _ in 0..count {
        let pos = s
            .cursor
            .unwrap_or_else(|| panic!("delete_at_caret 前无光标（go_to_marker）"));
        edit_script(s, pos, pos + 1, "");
    }
}

/// Go Backspace：删除光标前一字符并回退光标
pub fn backspace(s: &mut Session, count: usize) {
    for _ in 0..count {
        let pos = s
            .cursor
            .unwrap_or_else(|| panic!("backspace 前无光标（go_to_marker）"))
            .checked_sub(1)
            .unwrap_or_else(|| panic!("backspace 越过文件头"));
        edit_script(s, pos, pos + 1, "");
        s.cursor = Some(pos);
    }
}

/// Go ReplaceLine：选中整行（不含换行符）后键入 text 替换
pub fn replace_line(s: &mut Session, line_index: usize, text: &str) {
    let content = s.file_content(&s.active_file).to_string();
    let mut line_starts: Vec<usize> = vec![0];
    for (i, ch) in content.chars().enumerate() {
        if ch == '\n' {
            line_starts.push(i + 1);
        }
    }
    let Some(&start) = line_starts.get(line_index) else {
        panic!("replace_line 行号越界: {line_index}");
    };
    let end = line_starts
        .get(line_index + 1)
        .map(|&next_start| next_start - 1)
        .unwrap_or(content.chars().count());
    edit_script(s, start, end, text);
    s.cursor = Some(start + text.chars().count());
}

fn edit_script(s: &mut Session, start: usize, end: usize, new_text: &str) {
    let file = s.active_file.clone();
    let content = s.file_content(&file).to_string();
    let b_start = byte_index_of_char(&content, start);
    let b_end = byte_index_of_char(&content, end);
    let mut next = String::with_capacity(content.len() + new_text.len());
    next.push_str(&content[..b_start]);
    next.push_str(new_text);
    next.push_str(&content[b_end..]);
    s.set_file_content(&file, next);
    shift_positions(s, &file, start, end, new_text);
}

/// Go updatePosition：<= start 不动；在编辑区间内置 1（无效）；
/// 之后平移 len(new_text)-(end-start)
fn shift_positions(s: &mut Session, file: &str, start: usize, end: usize, new_text: &str) {
    let new_len: usize = new_text.chars().count();
    let delta = new_len as isize - (end as isize - start as isize);
    let shift = |p: &mut usize| {
        if *p <= start {
            // 不动
        } else if *p < end {
            *p = usize::MAX; // invalid
        } else {
            *p = (*p as isize + delta) as usize;
        }
    };
    for m in &mut s.data.markers {
        if m.file_name == file {
            shift(&mut m.position);
        }
    }
    for r in &mut s.data.ranges {
        if r.file_name == file {
            shift(&mut r.start);
            shift(&mut r.end);
        }
    }
    s.data.marker_positions = s
        .data
        .markers
        .iter()
        .filter(|m| m.position != usize::MAX)
        .filter_map(|m| m.name.clone().map(|n| (n, m.position)))
        .collect();
}

/// Go applyTextEdits：按起点升序排序后逆序应用，同步光标；
/// 返回净偏移（带符号，删除多于插入可为负）。
fn apply_text_edits(s: &mut Session, edits: &[crate::lsp::lsproto_lsp::TextEdit]) -> isize {
    let file = s.active_file.clone();
    let content = s.file_content(&file).to_string();
    let mut starts: Vec<(usize, usize, &str)> = edits
        .iter()
        .map(|e| {
            let start = line_col_to_offset(&content, e.range.start.line as usize, e.range.start.character as usize);
            let end = line_col_to_offset(&content, e.range.end.line as usize, e.range.end.character as usize);
            (start, end, e.new_text.as_str())
        })
        .collect();
    // LSP character 为 UTF-16；line_col_to_offset 按 ASCII 处理，
    // 用字符下标换算保持与 marker 体系一致
    starts.sort_by_key(|(start, _, _)| *start);

    let mut caret = s.cursor.unwrap_or(0);
    let mut edits_char: Vec<(usize, usize, String)> = starts
        .into_iter()
        .map(|(b_start, b_end, text)| {
            (
                char_index_of_byte(&content, b_start),
                char_index_of_byte(&content, b_end),
                text.to_string(),
            )
        })
        .collect();

    let mut total_offset: isize = 0;
    let mut buffer = content.clone();
    for (start, end, text) in edits_char.iter().rev() {
        // 逆序（高位置先拼接）下，低位置编辑的 char/byte 下标不受
        // 先前拼接影响，可在同一缓冲上按原始坐标顺序完成全部拼接；
        // marker/光标平移与拼接同序（降序），全部完成后只触发一次
        // 文件内容更新与服务重建
        let b_start = byte_index_of_char(&buffer, *start);
        let b_end = byte_index_of_char(&buffer, *end);
        buffer.replace_range(b_start..b_end, text);
        shift_positions(s, &file, *start, *end, text);
        let delta: isize = text.chars().count() as isize - (*end as isize - *start as isize);
        if *start <= caret {
            if *end <= caret {
                caret = (caret as isize + delta) as usize;
            } else {
                caret = *start;
            }
        }
        total_offset += delta;
    }
    if buffer != content {
        s.set_file_content(&file, buffer);
    }
    s.cursor = Some(caret);
    total_offset
}

fn on_type_formatting(
    s: &mut Session,
    offset: usize,
    ch: &str,
) -> Vec<crate::lsp::lsproto_lsp::TextEdit> {
    let uri = uri_of(s, &s.active_file);
    let service = match s.service.as_ref() {
        Some(service) => service,
        None => return Vec::new(),
    };
    let content = s.file_content(&s.active_file);
    let (line, character) = line_and_character(content, offset);
    let options = crate::lsp::lsproto_lsp::FormattingOptions::default();
    service.provide_format_document_on_type(
        &uri,
        &options,
        crate::lsp::lsproto_lsp::Position { line, character },
        ch,
    )
}

fn range_formatting(
    s: &mut Session,
    start: usize,
    end: usize,
) -> Vec<crate::lsp::lsproto_lsp::TextEdit> {
    let uri = uri_of(s, &s.active_file);
    let service = match s.service.as_ref() {
        Some(service) => service,
        None => return Vec::new(),
    };
    let content = s.file_content(&s.active_file);
    let (line0, col0) = line_and_character(content, start);
    let (line1, col1) = line_and_character(content, end);
    let options = crate::lsp::lsproto_lsp::FormattingOptions::default();
    service.provide_format_document_range(
        &uri,
        &options,
        crate::lsp::lsproto_lsp::Range {
            start: crate::lsp::lsproto_lsp::Position { line: line0, character: col0 },
            end: crate::lsp::lsproto_lsp::Position { line: line1, character: col1 },
        },
    )
}

fn char_index_of_byte(text: &str, byte: usize) -> usize {
    text[..byte.min(text.len())].chars().count()
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
            if d.severity.unwrap_or(0) == 1 {
                eprintln!(
                    "[no-err] {} line={}:{} code={:?} {}",
                    f.file_name,
                    d.range.start.line,
                    d.range.start.character,
                    d.code,
                    d.message
                );
            }
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
    let all = diagnostics_of(s, &s.active_file);
    let n = all.iter().filter(|d| d.severity.unwrap_or(0) == 1).count();
    if n != expected {
        for d in &all {
            eprintln!("[diag] sev={} code={:?} line={}:{}..{}: {}", d.severity.unwrap_or(0), d.code, d.range.start.line, d.range.start.character, d.range.end.character, d.message);
        }
        eprintln!("[file-content] {:?}", s.file_content(&s.active_file));
    }
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
    apply_text_edits_lsp(s, &edits);
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
    apply_text_edits_lsp(s, &edits);
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

#[doc(hidden)]
pub fn debug_dump_diagnostics(s: &Session, file: &str) {
    let ds = diagnostics_of_pub(s, file);
    for d in &ds {
        eprintln!("diag sev={:?} msg={}", d.severity, d.message);
    }
    eprintln!("count={}", ds.len());
}

fn diagnostics_of_pub(s: &Session, file: &str) -> Vec<crate::ls::types::Diagnostic> {
    diagnostics_of(s, file)
}
