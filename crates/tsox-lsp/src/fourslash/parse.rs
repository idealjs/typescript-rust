//! fourslash 测试内容的解析（纯函数）：多文件切分、标记/范围/对象标记提取。

use std::collections::BTreeMap;

pub const FILENAME_DIRECTIVE: &str = "@Filename:";
pub const SYMLINK_DIRECTIVE: &str = "@SYMlink:";
pub const GLOBAL_OPTIONS_DIRECTIVE: &str = "@GlobalOptions:";

fn strip_directive<'a>(rest: &'a str, directive: &str) -> Option<&'a str> {
    let rb = rest.as_bytes();
    let db = directive.as_bytes();
    if rb.len() >= db.len() && rb[..db.len()].eq_ignore_ascii_case(db) {
        Some(&rest[db.len()..])
    } else {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Marker {
    pub file_name: String,
    pub position: usize,
    pub name: Option<String>,
    pub data: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeMarker {
    pub file_name: String,
    pub start: usize,
    pub end: usize,
    pub marker: Option<Marker>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestFileInfo {
    pub file_name: String,
    pub content: String,
    pub file_options: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TestData {
    pub files: Vec<TestFileInfo>,
    pub markers: Vec<Marker>,
    pub marker_positions: BTreeMap<String, usize>,
    pub ranges: Vec<RangeMarker>,
    pub global_options: BTreeMap<String, String>,
    pub symlinks: BTreeMap<String, String>,
}

struct FileAccumulator {
    name: Option<String>,
    options: BTreeMap<String, String>,
    symlink: Option<String>,
    lines: Vec<String>,
}

impl FileAccumulator {
    fn new() -> Self {
        Self {
            name: None,
            options: BTreeMap::new(),
            symlink: None,
            lines: Vec::new(),
        }
    }

    fn is_empty(&self) -> bool {
        self.name.is_none() && self.lines.is_empty() && self.symlink.is_none()
    }

    fn finish(mut self, default_name: &str, data: &mut TestData) {
        if self.is_empty() {
            // 无内容无文件名的头部选项行（首个 @Filename 前的 // @option）按
            // Go fourslash 语义并入全项目选项
            for (k, v) in self.options {
                data.global_options.entry(k).or_insert(v);
            }
            return;
        }
        let name = self.name.take().unwrap_or_else(|| default_name.to_string());
        let joined = self.lines.join("\n");
        let (content, head_options, markers, ranges) = parse_file_content(&name, &joined);
        for (k, v) in head_options {
            self.options.entry(k).or_insert(v);
        }
        if let Some(l) = self.symlink.take() {
            data.symlinks.insert(name.clone(), l);
        }
        data.markers.extend(markers);
        data.ranges.extend(ranges);
        data.files.push(TestFileInfo {
            file_name: name,
            content,
            file_options: self.options,
        });
    }
}

pub fn parse_test_data(contents: &str, default_file_name: &str) -> TestData {
    let contents = contents.replace("\r\n", "\n");
    let mut data = TestData::default();
    let mut acc = FileAccumulator::new();

    for line in contents.split('\n') {
        let trimmed = line.trim_start();
        // 指令形态两种：`// @Filename:` 与 `//@Filename:`（tsc fourslash 均合法）
        let rest_after_slashes = trimmed.strip_prefix("//").map(|r| r.trim_start_matches(' '));
        if let Some(rest) = rest_after_slashes {
            if let Some(f) = strip_directive(rest, FILENAME_DIRECTIVE) {
                acc.finish(default_file_name, &mut data);
                acc = FileAccumulator::new();
                acc.name = Some(f.trim().to_string());
                continue;
            }
            if let Some(s) = strip_directive(rest, SYMLINK_DIRECTIVE) {
                acc.symlink = Some(s.trim().to_string());
                continue;
            }
            if let Some(g) = strip_directive(rest, GLOBAL_OPTIONS_DIRECTIVE) {
                for kv in g.split(',') {
                    if let Some((k, v)) = kv.split_once(':') {
                        data.global_options
                            .insert(k.trim().to_string(), v.trim().to_string());
                    }
                }
                continue;
            }
            if rest.starts_with('@') {
                let body = &rest[1..];
                if let Some((k, v)) = body.split_once(':') {
                    acc.options
                        .insert(k.trim().to_lowercase(), v.trim().to_string());
                } else {
                    acc.options
                        .insert(body.trim().to_lowercase(), "true".into());
                }
                continue;
            }
        }
        acc.lines.push(line.to_string());
    }
    acc.finish(default_file_name, &mut data);

    data.marker_positions = data
        .markers
        .iter()
        .filter_map(|m| m.name.clone().map(|n| (n, m.position)))
        .collect();
    data
}

fn parse_file_content(
    file_name: &str,
    content: &str,
) -> (
    String,
    BTreeMap<String, String>,
    Vec<Marker>,
    Vec<RangeMarker>,
) {
    let chars: Vec<char> = content.chars().collect();
    let mut out = String::new();
    let mut markers: Vec<Marker> = Vec::new();
    let mut ranges: Vec<RangeMarker> = Vec::new();
    let mut open_ranges: Vec<(usize, Option<Marker>)> = Vec::new();
    let mut difference: usize = 0;
    let mut open_source: Option<(usize, bool)> = None; // (源位置, 是否对象标记)
    let mut last_normal = 0usize;
    let mut state = 0u8; // 0 普通, 1 slash-star 标记, 2 对象标记

    fn flush(out: &mut String, chars: &[char], from: &mut usize, to: Option<usize>) {
        let end = to.unwrap_or(chars.len()).min(chars.len());
        if *from < end {
            out.extend(chars[*from..end].iter());
        }
        if to.is_some() {
            *from = to.unwrap();
        }
    }

    if chars.is_empty() {
        return (out, BTreeMap::new(), markers, ranges);
    }
    let mut prev = chars[0];
    for i in 1..chars.len() {
        let cur = chars[i];
        match state {
            0 => {
                if prev == '[' && cur == '|' {
                    open_ranges.push((i - 1 - difference, None));
                    flush(&mut out, &chars, &mut last_normal, Some(i - 1));
                    last_normal = i + 1;
                    difference += 2;
                } else if prev == '|' && cur == ']' {
                    let Some((start, marker)) = open_ranges.pop() else {
                        panic!("{file_name}: range 结束无起始");
                    };
                    ranges.push(RangeMarker {
                        file_name: file_name.into(),
                        start,
                        end: i - 1 - difference,
                        marker,
                    });
                    flush(&mut out, &chars, &mut last_normal, Some(i - 1));
                    last_normal = i + 1;
                    difference += 2;
                } else if prev == '/' && cur == '*' && chars.get(i + 1) != Some(&'/') {
                    state = 1;
                    open_source = Some((i - 1, false));
                } else if prev == '{' && cur == '|' {
                    state = 2;
                    open_source = Some((i - 1, true));
                    flush(&mut out, &chars, &mut last_normal, Some(i - 1));
                }
            }
            2 => {
                if prev == '|' && cur == '}' {
                    let (src, _) = open_source.take().unwrap();
                    let text: String = chars[src + 2..i - 1].iter().collect();
                    let data = text.trim().to_string();
                    // Go getObjectMarker：按 JSON 解析，"name" 字段可用作标记名
                    let name = serde_json::from_str::<serde_json::Value>(&format!("{{ {data} }}"))
                        .ok()
                        .and_then(|v| {
                            v.get("name")
                                .and_then(|n| n.as_str())
                                .filter(|n| !n.is_empty())
                                .map(str::to_string)
                        });
                    markers.push(Marker {
                        file_name: file_name.into(),
                        position: src - difference,
                        name,
                        data: Some(data),
                    });
                    if let Some(last) = open_ranges.last_mut() {
                        last.1 = markers.last().cloned();
                    }
                    flush(&mut out, &chars, &mut last_normal, Some(i + 1));
                    difference += i + 1 - src;
                    state = 0;
                }
            }
            _ => {
                if prev == '*' && cur == '/' {
                    let (src, _) = open_source.take().unwrap();
                    let text: String = chars[src + 2..i - 1].iter().collect();
                    markers.push(Marker {
                        file_name: file_name.into(),
                        position: src - difference,
                        name: Some(text.trim().to_string()),
                        data: None,
                    });
                    if let Some(last) = open_ranges.last_mut() {
                        last.1 = markers.last().cloned();
                    }
                    flush(&mut out, &chars, &mut last_normal, Some(src));
                    last_normal = i + 1;
                    difference += i + 1 - src;
                    state = 0;
                } else if !(cur.is_ascii_alphanumeric() || cur == '$' || cur == '_')
                    && !(cur == '*' && chars.get(i + 1) == Some(&'/'))
                {
                    // 非法标记字符：实为普通块注释
                    flush(&mut out, &chars, &mut last_normal, Some(i));
                    open_source = None;
                    state = 0;
                }
            }
        }
        prev = cur;
    }
    flush(&mut out, &chars, &mut last_normal, None);
    assert!(open_ranges.is_empty(), "{file_name}: range 未闭合");

    let mut file_options = BTreeMap::new();
    if let Some(first) = out.lines().next() {
        if let Some(rest) = first.trim().strip_prefix("// @") {
            if let Some((k, v)) = rest.split_once(':') {
                file_options.insert(k.trim().to_lowercase(), v.trim().to_string());
            }
        }
    }
    (out, file_options, markers, ranges)
}
