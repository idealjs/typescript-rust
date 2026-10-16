use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TestUnit {

    pub name: String,

    pub content: String,
}

#[derive(Debug, Clone)]
pub struct ParsedCase {

    pub units: Vec<TestUnit>,

    pub settings: HashMap<String, String>,

    pub current_directory: Option<String>,

    /// `// @symlink: p1,p2`（Go 语义：link 路径 → 出现处的当前 unit 文件）
    pub symlinks: Vec<(String, String)>,
}

fn parse_directive_line(line: &str) -> Option<(&str, &str)> {

    let line = line.trim_start();
    let after_slashes = line.strip_prefix("//")?;

    let after_at = after_slashes.trim_start().strip_prefix('@')?;

    let colon = after_at.find(':')?;
    let name = after_at[..colon].trim();
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return None;
    }
    let value = after_at[colon + 1..].trim();
    Some((name, value))
}

pub fn extract_settings(content: &str) -> HashMap<String, String> {

    const HARNESS_DIRECTIVES: &[&str] = &[
        "notypesandsymbols",
        "noimplicitreferences",
        "fullemitpaths",
        "traceresolution",
        "baselinefile",
        "libfiles",
        "reportdiagnostics",
        "capturesuggestions",
        "typescriptversion",
        "emitthisfile",
        "currentdirectory",
        "symlink",
        "link",
        "filename",
    ];
    let mut map = HashMap::new();

    let content = content.strip_prefix('\u{feff}').unwrap_or(content);
    for line in content.lines() {
        if let Some((name, value)) = parse_directive_line(line) {
            let lower = name.to_ascii_lowercase();
            if HARNESS_DIRECTIVES.contains(&lower.as_str()) {
                continue;
            }
            let v = value.trim_end_matches(';').trim().to_string();
            map.entry(lower).or_insert(v);
        }
    }
    map
}

pub fn split_units(content: &str, default_name: &str) -> ParsedCase {

    let content = content.strip_prefix('\u{feff}').unwrap_or(content);
    let mut units: Vec<TestUnit> = Vec::new();
    let mut settings: HashMap<String, String> = HashMap::new();
    let mut current_directory: Option<String> = None;

    let mut current_name: Option<String> = None;
    let mut current_body = String::new();
    let mut symlinks: Vec<(String, String)> = Vec::new();

    let flush = |units: &mut Vec<TestUnit>, name: &mut Option<String>, body: &mut String| {
        if let Some(n) = name.take() {
            units.push(TestUnit {
                name: n,
                content: std::mem::take(body),
            });
        }
    };

    for line in content.split('\n') {
        let line_no_nl = line.trim_end_matches('\r');
        if let Some((name, value)) = parse_directive_line(line_no_nl) {
            let lower = name.to_ascii_lowercase();
            match lower.as_str() {
                "filename" => {

                    if current_name.is_some() {
                        flush(&mut units, &mut current_name, &mut current_body);
                    }

                    current_name = Some(value.to_string());
                    current_body.clear();
                }
                "currentdirectory" => {
                    current_directory = Some(value.to_string());
                }
                "symlink" => {
                    if let Some(n) = &current_name {
                        for link in value.split(',') {
                            let link = link.trim();
                            if !link.is_empty() {
                                symlinks.push((link.to_string(), n.clone()));
                            }
                        }
                    }
                }
                // Go harnessutil @link：<源> -> <链接路径>，源可为目录前缀
                "link" => {
                    if let Some((src, dst)) = value.split_once("->") {
                        let src = src.trim();
                        let dst = dst.trim();
                        if !src.is_empty() && !dst.is_empty() {
                            symlinks.push((dst.to_string(), src.to_string()));
                        }
                    }
                }
                _ => {

                    settings
                        .entry(lower)
                        .or_insert_with(|| value.trim_end_matches(';').trim().to_string());
                }
            }
            continue;
        }

        if current_name.is_some() {

            if current_body.is_empty() && line_no_nl.is_empty() {
                continue;
            }
            // Go ParseTestFilesAndSymlinksWithOptions（test_case_parser.go:19
            // lineDelimiter=/\r?\n/ 切行丢终止符、248-265 以 '\n' 重拼，末尾
            // 空行片同样写入）：单元内容行尾形态与 Go Split 逐片一致
            if !current_body.is_empty() {
                current_body.push('\n');
            }
            current_body.push_str(line_no_nl);
        }
    }

    if current_name.is_some() {
        flush(&mut units, &mut current_name, &mut current_body);
    } else {

        let mut body = String::new();
        for line in content.split('\n') {
            let line_no_nl = line.trim_end_matches('\r');
            if parse_directive_line(line_no_nl).is_some() {
                continue;
            }
            if body.is_empty() && line_no_nl.is_empty() {
                continue;
            }
            if !body.is_empty() {
                body.push('\n');
            }
            body.push_str(line_no_nl);
        }
        units.push(TestUnit {
            name: default_name.to_string(),
            content: body,
        });
    }

    ParsedCase {
        units,
        settings,
        current_directory,
        symlinks,
    }
}
