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
