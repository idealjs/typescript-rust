//! LSP position 与源文件字节 offset 的唯一换算点：character 按 UTF-16 码元推进。

use tsox_frontend::ast::node::LineMap;

pub(crate) fn lsp_position_to_offset(
    text: &str,
    line_map: &LineMap,
    line: usize,
    character: usize,
) -> usize {
    let line_start = line_map.line_starts.get(line).copied().unwrap_or(0) as usize;
    let mut offset = line_start;
    let mut units = 0usize;
    for c in text[line_start..].chars() {
        if units >= character {
            break;
        }
        units += c.len_utf16();
        offset += c.len_utf8();
    }
    offset
}

pub(crate) fn offset_to_lsp_line_char(
    text: &str,
    line_map: &LineMap,
    offset: usize,
) -> (usize, usize) {
    let line = match line_map.line_starts.binary_search(&(offset as u32)) {
        Ok(idx) => idx,
        Err(idx) => idx.saturating_sub(1),
    };
    let line_start = line_map.line_starts.get(line).copied().unwrap_or(0) as usize;
    (line, line_map.utf16_column_at(text, offset) - line_map.utf16_column_at(text, line_start))
}
