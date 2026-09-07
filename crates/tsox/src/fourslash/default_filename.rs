use std::sync::Arc;

use crate::ast::{Node, SourceFile};
use crate::compiler::{CompilerHost, CompilerHostImpl, Program, ProgramOptions};
use crate::core::tristate::Tristate;
use crate::tsoptions::ParsedCommandLine;
use crate::vfs::InMemoryFS;

use crate::bundled::lib_path;

pub(crate) const DEFAULT_FILENAME: &str = "/proj/fourslash.ts";

#[derive(Debug, Clone)]
pub struct Marker {
    pub name: String,

    pub position: usize,
}

#[derive(Debug, Clone)]
pub struct RangeMarker {
    pub name: String,

    pub start: usize,

    pub end: usize,
}

#[derive(Debug, Clone)]
pub struct ParsedFile {
    pub filename: String,
    pub content: String,
    pub markers: Vec<Marker>,
    pub ranges: Vec<RangeMarker>,
}

pub fn parse_test_content(content: &str) -> Vec<ParsedFile> {
    let mut files: Vec<(String, String)> = Vec::new();
    let mut current_name = DEFAULT_FILENAME.to_string();
    let mut current_body = String::new();
    let mut started = false;

    for line in content.split_inclusive('\n') {
        if let Some(name) = parse_filename_directive(line.trim_start()) {
            if started {
                files.push((current_name, std::mem::take(&mut current_body)));
            }
            current_name = normalize_filename(name);
            started = true;
            continue;
        }
        started = true;
        current_body.push_str(line);
    }
    files.push((current_name, current_body));

    files
        .into_iter()
        .map(|(filename, body)| {
            let (content, markers, ranges) = strip_markers(&body);
            ParsedFile {
                filename,
                content,
                markers,
                ranges,
            }
        })
        .collect()
}

pub(crate) fn parse_filename_directive(line: &str) -> Option<&str> {
    let line = line.strip_prefix("//")?.trim_start();
    let line = line.strip_prefix("@")?;

    let split_at = 8.min(line.len());
    let (kw, rest) = line.split_at(split_at);
    if !kw.eq_ignore_ascii_case("filename") {
        return None;
    }
    let rest = rest.trim_start();
    let rest = rest.strip_prefix(":")?;
    Some(rest.trim())
}

pub(crate) fn normalize_filename(name: &str) -> String {
    let name = name.trim();
    if name.starts_with('/') {
        name.to_string()
    } else {
        format!("/proj/{name}")
    }
}

pub(crate) fn strip_markers(raw: &str) -> (String, Vec<Marker>, Vec<RangeMarker>) {
    let mut cleaned = String::with_capacity(raw.len());
    let mut markers = Vec::new();
    let mut ranges = Vec::<RangeMarker>::new();
    let mut i = 0;

    while i < raw.len() {
        let rest = &raw[i..];

        if rest.starts_with("/**/") {
            markers.push(Marker {
                name: String::new(),
                position: cleaned.len(),
            });
            i += 4;
            continue;
        }

        if rest.starts_with("/*") {
            if let Some(close) = rest.find("*/") {
                let inner = &rest[2..close];
                if is_marker_name(inner) {
                    markers.push(Marker {
                        name: inner.to_string(),
                        position: cleaned.len(),
                    });
                    i += close + 2;
                    continue;
                }
            }

            cleaned.push('/');
            i += 1;
            continue;
        }

        if let Some(inner_part) = rest.strip_prefix("[|") {
            if let Some(close) = inner_part.find("|]") {
                let inner = &inner_part[..close];
                let (name, content) = match inner.find('|') {
                    Some(p) => (inner[..p].to_string(), &inner[p + 1..]),
                    None => (String::new(), inner),
                };
                let start = cleaned.len();
                cleaned.push_str(content);
                let end = cleaned.len();
                ranges.push(RangeMarker { name, start, end });

                i += 2 + close + 2;
                continue;
            }

            cleaned.push('[');
            i += 1;
            continue;
        }

        let ch = rest.chars().next().unwrap();
        cleaned.push(ch);
        i += ch.len_utf8();
    }

    (cleaned, markers, ranges)
}

pub(crate) fn is_marker_name(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

pub struct FourslashTest {
    pub files: Vec<ParsedFile>,
}

impl FourslashTest {
    pub fn new(content: &str) -> Self {
        Self {
            files: parse_test_content(content),
        }
    }

    pub fn get_marker(&self, name: &str) -> usize {
        for f in &self.files {
            for m in &f.markers {
                if name.is_empty() {
                    if m.name.is_empty() {
                        return m.position;
                    }
                } else if m.name == name {
                    return m.position;
                }
            }
        }
        panic!("fourslash marker not found: {name:?}");
    }

    pub fn get_file(&self, filename: &str) -> &ParsedFile {
        self.files
            .iter()
            .find(|f| f.filename == filename || basename(&f.filename) == filename)
            .unwrap_or_else(|| panic!("fourslash file not found: {filename}"))
    }

    pub fn build_program(&self) -> Arc<Program> {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        let mut file_names = Vec::with_capacity(self.files.len());
        for f in &self.files {
            if let Some(parent) = std::path::Path::new(&f.filename).parent() {
                if !parent.as_os_str().is_empty() {
                    fs.insert_dir(&parent.to_string_lossy());
                }
            }
            fs.insert_file(&f.filename, &f.content);
            file_names.push(f.filename.clone());
        }

        let host = CompilerHostImpl::new(fs, "/proj".to_string(), lib_path());
        let host: Arc<dyn CompilerHost> = Arc::new(host);

        let mut config = ParsedCommandLine::default();
        config.file_names = file_names;
        config.compiler_options.no_lib = Tristate::True;

        Arc::new(Program::new(ProgramOptions { config, host }))
    }

    pub fn hover_at(&self, file: &ParsedFile, offset: usize) -> String {
        let program = self.build_program();
        let sf = program
            .get_source_file(&file.filename)
            .unwrap_or_else(|| panic!("source file not found: {}", file.filename));
        let node = deepest_node_at(&sf, offset);
        let mut checker = program.build_checker();
        let parts = checker.get_quick_info_display_parts(&node);
        if parts.is_empty() {
            checker.get_quick_info_text(&node)
        } else {
            parts.iter().map(|p| p.text.as_str()).collect()
        }
    }

    pub fn completions_at(&self, file: &ParsedFile, _offset: usize) -> Vec<String> {
        let program = self.build_program();
        let _sf = program
            .get_source_file(&file.filename)
            .unwrap_or_else(|| panic!("source file not found: {}", file.filename));
        let checker = program.build_checker();
        let mut labels: Vec<String> = checker
            .globals
            .iter()
            .filter(|(name, _)| !name.starts_with('\u{FE}') && !name.starts_with("__"))
            .map(|(name, _)| name.clone())
            .collect();
        labels.sort();
        labels
    }

    pub fn definition_at(&self, file: &ParsedFile, offset: usize) -> Option<(String, usize)> {
        let program = self.build_program();
        let sf = program
            .get_source_file(&file.filename)
            .unwrap_or_else(|| panic!("source file not found: {}", file.filename));
        let node = deepest_node_at(&sf, offset);
        let checker = program.build_checker();

        let symbol = checker.resolve_identifier(&node).or_else(|| {
            let symbol_map = checker.program.symbol_map();
            let mut current: Option<&Arc<Node>> = Some(&node);
            while let Some(n) = current {
                if let Some(sym) = symbol_map.symbol_of(n) {
                    return Some(Arc::clone(sym));
                }
                current = n.parent.as_ref();
            }
            None
        })?;

        let decl = symbol.value_declaration.as_ref()?;

        for sf in program.source_files() {
            if decl.pos() < sf.text.len() {
                return Some((sf.file_name.clone(), decl.pos()));
            }
        }
        None
    }
}

pub(crate) fn deepest_node_at(sf: &Arc<SourceFile>, offset: usize) -> Arc<Node> {
    crate::astnav::get_token_at_position(&sf.node, offset).unwrap_or_else(|| Arc::clone(&sf.node))
}

pub(crate) fn basename(path: &str) -> &str {
    match path.rsplit_once('/') {
        Some((_, base)) => base,
        None => path,
    }
}
