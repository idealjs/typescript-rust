use std::sync::Arc;

use crate::ast::{Node, SourceFile};
use crate::compiler::{CompilerHost, CompilerHostImpl, Program, ProgramOptions};
use crate::core::tristate::Tristate;
use crate::tsoptions::ParsedCommandLine;
use crate::vfs::InMemoryFS;

use crate::bundled::lib_path;

const DEFAULT_FILENAME: &str = "/proj/fourslash.ts";

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

fn parse_filename_directive(line: &str) -> Option<&str> {
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

fn normalize_filename(name: &str) -> String {
    let name = name.trim();
    if name.starts_with('/') {
        name.to_string()
    } else {
        format!("/proj/{name}")
    }
}

fn strip_markers(raw: &str) -> (String, Vec<Marker>, Vec<RangeMarker>) {
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

fn is_marker_name(s: &str) -> bool {
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

fn deepest_node_at(sf: &Arc<SourceFile>, offset: usize) -> Arc<Node> {
    crate::astnav::get_token_at_position(&sf.node, offset).unwrap_or_else(|| Arc::clone(&sf.node))
}

fn basename(path: &str) -> &str {
    match path.rsplit_once('/') {
        Some((_, base)) => base,
        None => path,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_anonymous_marker() {
        let t = FourslashTest::new("function foo() {}");
        let f = &t.files[0];
        assert_eq!(f.content, "function foo() {}");
        assert!(f.markers.is_empty());
        assert_eq!(f.filename, DEFAULT_FILENAME);
    }

    #[test]
    fn test_parse_marker_offsets() {

        let t = FourslashTest::new("function /**/foo(): number { return 1; }");
        let f = &t.files[0];
        assert_eq!(f.content, "function foo(): number { return 1; }");
        assert_eq!(f.markers.len(), 1);
        let pos = t.get_marker("");
        assert_eq!(&f.content[pos..pos + 3], "foo");
    }

    #[test]
    fn test_named_markers() {
        let t = FourslashTest::new("let /*a*/x = 1; let /*b*/y = 2;");
        let f = &t.files[0];
        assert_eq!(f.content, "let x = 1; let y = 2;");
        let a = t.get_marker("a");
        let b = t.get_marker("b");
        assert_eq!(&f.content[a..a + 1], "x");
        assert_eq!(&f.content[b..b + 1], "y");
    }

    #[test]
    fn test_range_markers() {
        let t = FourslashTest::new("const s = [|sel|world|];");
        let f = &t.files[0];
        assert_eq!(f.content, "const s = world;");
        assert_eq!(f.ranges.len(), 1);
        let r = &f.ranges[0];
        assert_eq!(r.name, "sel");
        assert_eq!(&f.content[r.start..r.end], "world");
    }

    #[test]
    fn test_filename_directive_splits_files() {
        let src =
            "// @filename: a.ts\nexport const shared = 1;\n// @filename: b.ts\nconst local = 2;";
        let t = FourslashTest::new(src);
        assert_eq!(t.files.len(), 2);
        let a = t.get_file("a.ts");
        let b = t.get_file("b.ts");
        assert!(a.content.contains("shared"));
        assert!(b.content.contains("local"));
    }

    #[test]
    fn test_hover_function() {
        let t = FourslashTest::new("function /**/foo(): number { return 1; }");
        let pos = t.get_marker("");
        let file = &t.files[0];
        let hover = t.hover_at(file, pos);
        assert!(hover.contains("foo"), "hover was: {hover:?}");
        assert!(hover.contains("number"), "hover was: {hover:?}");
    }

    #[test]
    fn test_hover_variable() {

        let t = FourslashTest::new("const /**/x = 42;");
        let pos = t.get_marker("");
        let file = &t.files[0];
        let hover = t.hover_at(file, pos);
        assert!(hover.contains("x"), "hover was: {hover:?}");
        assert!(hover.contains("42"), "hover was: {hover:?}");
    }

    #[test]
    fn test_completion_basic() {
        let t = FourslashTest::new("const alpha = 1;\nconst beta = 2;\n/**/\n");
        let pos = t.get_marker("");
        let file = &t.files[0];
        let labels = t.completions_at(file, pos);
        assert!(labels.iter().any(|l| l == "alpha"), "labels: {labels:?}");
        assert!(labels.iter().any(|l| l == "beta"), "labels: {labels:?}");
    }

    #[test]
    fn test_definition() {
        let t = FourslashTest::new(
            "function greet(): string { return \"hi\"; }\nconst s = gr/**/eet();",
        );
        let pos = t.get_marker("");
        let file = &t.files[0];
        let (fname, offset) = t
            .definition_at(file, pos)
            .expect("definition should resolve");
        assert_eq!(fname, file.filename);

        assert_eq!(offset, 0, "definition offset");
    }

    #[test]
    fn test_multi_file_program() {
        let src =
            "// @filename: a.ts\nexport const shared = 1;\n// @filename: b.ts\nconst local = 2;";
        let t = FourslashTest::new(src);
        assert_eq!(t.files.len(), 2);

        let program = t.build_program();
        assert_eq!(program.source_files().len(), 2);
        assert!(program.get_source_file("/proj/a.ts").is_some());
        assert!(program.get_source_file("/proj/b.ts").is_some());
    }
}
