use std::sync::Arc;

use tsox_compile::compiler::Program;
use tsox_frontend::ast::SourceFile;

use super::language_service::LanguageService;

/// Go parseTripleSlashDirectiveFragment：`/// <reference path|types="fragment`
/// 形式（fragment 未闭合），返回 (kind, toComplete)
fn parse_triple_slash_fragment(text: &str) -> Option<(&'static str, &str)> {
    let rest = text.strip_prefix("///")?;
    let rest = rest.trim_start();
    if !rest.starts_with('<') {
        return None;
    }
    let rest = rest.strip_prefix('<')?;
    let rest = rest.strip_prefix("reference")?;
    if !rest.starts_with(|c: char| c.is_whitespace()) {
        return None;
    }
    let rest = rest.trim_start();
    let (kind, rest) = if let Some(r) = rest.strip_prefix("path") {
        ("path", r)
    } else if let Some(r) = rest.strip_prefix("types") {
        ("types", r)
    } else {
        return None;
    };
    let rest = rest.trim_start();
    let rest = rest.strip_prefix('=')?;
    let rest = rest.trim_start();
    let rest = rest.strip_prefix(|c: char| c == '"' || c == '\'')?;
    if rest.contains(['"', '\'']) {
        return None;
    }
    Some((kind, rest))
}

fn dir_of(file_name: &str) -> String {
    match file_name.rfind('/') {
        Some(0) => "/".to_string(),
        Some(i) => file_name[..i].to_string(),
        None => String::new(),
    }
}

fn normalize_path(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => continue,
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    format!("/{}", parts.join("/"))
}

/// Go getTripleSlashReferenceCompletions：`/// <reference path|types="…`
/// 注释内（引号未闭合）的路径 / 包名补全
pub fn triple_slash_reference_labels(
    service: &LanguageService,
    program: &Arc<Program>,
    file: &SourceFile,
    text: &str,
    position: usize,
) -> Option<Vec<String>> {
    let pos = position.min(text.len());
    let line_start = text[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let (kind, to_complete) = parse_triple_slash_fragment(&text[line_start..pos])?;
    let script_dir = dir_of(&file.file_name);

    match kind {
        "path" => {
            let directory = match to_complete.rfind('/') {
                Some(idx) => &to_complete[..=idx],
                None => "./",
            };
            // 绝对路径说明符（`/tests/…`）不拼 script_dir
            let base_dir = if to_complete.starts_with('/') {
                normalize_path(directory)
            } else {
                normalize_path(&format!("{script_dir}/{directory}"))
            };
            let prefix = if base_dir == "/" {
                "/".to_string()
            } else {
                format!("{base_dir}/")
            };
            let mut labels: Vec<String> = Vec::new();
            // Go ReadDirectory 扩展过滤：allowJs=false 时只枚举 TS 扩展
            // （SupportedTSExtensions）
            let ts_exts = [
                ".ts", ".tsx", ".d.ts", ".mts", ".cts", ".d.mts", ".d.cts",
            ];
            let js_exts = [".js", ".jsx", ".mjs", ".cjs"];
            for other in program.source_files() {
                if other.file_name == file.file_name {
                    continue;
                }
                let Some(rest) = other.file_name.strip_prefix(&prefix) else {
                    continue;
                };
                if rest.is_empty() {
                    continue;
                }
                // reference path 是文件位：文件名保留扩展名，目录取首段
                let label = match rest.find('/') {
                    Some(idx) => rest[..idx].to_string(),
                    None => rest.to_string(),
                };
                if let Some(ext) = label.rfind('.')
                    && !ts_exts.contains(&&label[ext..])
                    && (!program.options().get_allow_js() || !js_exts.contains(&&label[ext..]))
                {
                    continue;
                }
                labels.push(label);
            }
            labels.sort();
            labels.dedup();
            Some(labels)
        }
        _ => {
            let project_root = service.project_path.to_string();
            let (type_roots, _) = tsox_tsoptions::module::resolver::get_effective_type_roots(
                program.options(),
                &project_root,
            );
            let mut labels: Vec<String> = Vec::new();
            for root in &type_roots {
                let root = if root.starts_with('/') {
                    root.clone()
                } else {
                    tsox_core::tspath::combine_paths(&project_root, &[root])
                };
                if !service.directory_exists(&root) {
                    continue;
                }
                for dir in service.get_directories(&root).iter() {
                    let base = tsox_core::tspath::get_base_file_name(dir);
                    if base.starts_with('.') || base == "@types" {
                        continue;
                    }
                    labels.push(base.to_string());
                }
            }
            labels.sort();
            labels.dedup();
            Some(labels)
        }
    }
}
