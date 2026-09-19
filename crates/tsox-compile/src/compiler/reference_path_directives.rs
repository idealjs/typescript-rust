#![allow(unused_imports)]

use super::*;

enum ScanMode {
    Code { depth: u32 },
    LineComment,
    BlockComment,
    SingleQuoted,
    DoubleQuoted,
    Template,
}

impl ScanMode {
    fn is_code(&self) -> bool {
        matches!(self, ScanMode::Code { .. })
    }
}

// Go scanner processComment 语义：三斜线指令只出自单行注释。
// 块注释/字符串/模板字面量内部的 /// 行不是注释，不产出指令。
// 行首扫描状态为 Code 的行才允许提取 directive。
pub(crate) fn code_line_mask(text: &str) -> Vec<bool> {
    let bytes = text.as_bytes();
    let mut mask = Vec::new();
    let mut stack: Vec<ScanMode> = vec![ScanMode::Code { depth: 0 }];
    let mut escaped = false;
    let mut at_line_start = true;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if at_line_start {
            mask.push(stack.last().map(ScanMode::is_code).unwrap_or(true));
            at_line_start = false;
        }
        let escaped_now = escaped;
        escaped = false;
        let mut consumed_next = false;
        let in_template_expr = stack.len() > 1;
        if escaped_now {
            // skip
        } else {
            match stack.last_mut().unwrap() {
                ScanMode::Code { depth } => match b {
                    b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                        stack.push(ScanMode::LineComment);
                        consumed_next = true;
                    }
                    b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                        stack.push(ScanMode::BlockComment);
                        consumed_next = true;
                    }
                    b'\'' => stack.push(ScanMode::SingleQuoted),
                    b'"' => stack.push(ScanMode::DoubleQuoted),
                    b'`' => stack.push(ScanMode::Template),
                    b'{' if in_template_expr => *depth += 1,
                    b'}' if in_template_expr => {
                        if *depth == 0 {
                            stack.pop();
                        } else {
                            *depth -= 1;
                        }
                    }
                    _ => {}
                },
                ScanMode::LineComment => {
                    if b == b'\n' {
                        stack.pop();
                    }
                }
                ScanMode::BlockComment => {
                    if b == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                        stack.pop();
                        consumed_next = true;
                    }
                }
                ScanMode::SingleQuoted | ScanMode::DoubleQuoted | ScanMode::Template => {
                    match b {
                        b'\\' => escaped = true,
                        b'\'' if matches!(stack.last(), Some(ScanMode::SingleQuoted)) => {
                            stack.pop();
                        }
                        b'"' if matches!(stack.last(), Some(ScanMode::DoubleQuoted)) => {
                            stack.pop();
                        }
                        b'`' if matches!(stack.last(), Some(ScanMode::Template)) => {
                            stack.pop();
                        }
                        b'$' if matches!(stack.last(), Some(ScanMode::Template))
                            && i + 1 < bytes.len()
                            && bytes[i + 1] == b'{' =>
                        {
                            stack.push(ScanMode::Code { depth: 0 });
                            consumed_next = true;
                        }
                        _ => {}
                    }
                }
            }
        }
        if b == b'\n' {
            at_line_start = true;
        }
        i += 1 + usize::from(consumed_next);
    }
    mask
}

pub(crate) struct ReferencePathDirective {
    pub(crate) resolved: String,
    pub(crate) raw: String,
    pub(crate) value_range: (usize, usize),
}

fn supported_reference_extensions() -> &'static [&'static str] {
    &[".ts", ".tsx", ".d.ts"]
}

pub(crate) fn extract_reference_path_directives(
    text: &str,
    containing_file: &str,
) -> Vec<ReferencePathDirective> {
    let mut refs = Vec::new();
    let base_dir = tsox_core::tspath::get_directory_path(containing_file);
    let mask = code_line_mask(text);
    let mut line_start = 0usize;
    for (idx, raw_line) in text.split('\n').enumerate() {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let trimmed = line.trim_start();
        let leading = line.len() - trimmed.len();
        let in_code = mask.get(idx).copied().unwrap_or(true);
        let Some(rest) = trimmed.strip_prefix("///") else {
            line_start += raw_line.len() + 1;
            continue;
        };
        if !in_code || !rest.trim_start().starts_with("<reference") {
            line_start += raw_line.len() + 1;
            continue;
        }
        let Some((start, marker_len)) = ["path=\"", "path='"]
            .iter()
            .find_map(|m| rest.find(*m).map(|i| (i, m.len())))
        else {
            line_start += raw_line.len() + 1;
            continue;
        };
        let after = &rest[start + marker_len..];
        let quote = rest[start + 5..].chars().next().unwrap_or('"');
        if let Some(end) = after.find(quote) {
            let path = &after[..end];
            let resolved = if tsox_core::tspath::is_rooted_disk_path(path) {
                tsox_core::tspath::normalize_path(path)
            } else {
                tsox_core::tspath::normalize_path(&tsox_core::tspath::combine_paths(
                    &base_dir,
                    &[path],
                ))
            };
            let value_start = line_start + leading + 3 + start + marker_len;
            refs.push(ReferencePathDirective {
                resolved,
                raw: path.to_string(),
                value_range: (value_start, value_start + end),
            });
        }
        line_start += raw_line.len() + 1;
    }
    refs
}

// Go fileloader getSourceFileFromReference：带扩展名时校验扩展名/存在性/
// 自引用；无扩展名时按 supportedExtensions 顺序探测并返回候选路径。
pub(crate) fn resolve_reference_path(
    host: &dyn CompilerHost,
    resolved: &str,
    raw: &str,
    containing_file: &str,
) -> Result<String, (tsox_core::diagnostics::Message, Vec<String>)> {
    use tsox_core::diagnostics::messages_generated as msg;
    let has_extension = resolved.rsplit('/').next().unwrap_or("").contains('.');
    let raw_normalized = raw.replace('\\', "/");
    if has_extension {
        if !supported_reference_extensions()
            .iter()
            .any(|ext| resolved.ends_with(ext))
        {
            if resolved.ends_with(".js") || resolved.ends_with(".jsx") {
                return Err((msg::FILE_0_IS_A_JAVASCRIPT_FILE_DID_YOU_MEAN_TO_ENABLE_THE_ALLOWJS_OPTION, vec![raw_normalized]));
            }
            let joined = supported_reference_extensions()
                .iter()
                .map(|e| format!("'{e}'"))
                .collect::<Vec<_>>()
                .join(", ");
            return Err((msg::FILE_0_HAS_AN_UNSUPPORTED_EXTENSION_THE_ONLY_SUPPORTED_EXTENSIONS_ARE_1, vec![raw_normalized, joined]));
        }
        if !host.fs().file_exists(resolved) {
            return Err((tsox_core::diagnostics::FILE_0_NOT_FOUND, vec![raw_normalized]));
        }
        let canonical = |p: &str| {
            p.rsplit('/')
                .next()
                .unwrap_or("")
                .to_ascii_lowercase()
        };
        if canonical(resolved) == canonical(containing_file) {
            return Err((msg::A_FILE_CANNOT_HAVE_A_REFERENCE_TO_ITSELF, Vec::new()));
        }
        return Ok(resolved.to_string());
    }
    for ext in supported_reference_extensions() {
        let candidate = format!("{resolved}{ext}");
        if host.fs().file_exists(&candidate) {
            return Ok(candidate);
        }
    }
    let joined = supported_reference_extensions()
        .iter()
        .map(|e| format!("'{e}'"))
        .collect::<Vec<_>>()
        .join(", ");
    Err((msg::COULD_NOT_RESOLVE_THE_PATH_0_WITH_THE_EXTENSIONS_COLON_1, vec![raw_normalized, joined]))
}
