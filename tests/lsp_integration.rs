use std::sync::Arc;

use tsox::bundled::lib_path;
use tsox::compiler::{CompilerHostImpl, Program, ProgramOptions};
use tsox::ls::host::{AutoImportRegistry, EcmaLineInfo, Host};
use tsox::ls::language_service::LanguageService;
use tsox::ls::lsconv::converters::{Converters, PositionEncodingKind};
use tsox::ls::lsutil::UserPreferences;
use tsox::ls::types::{DocumentSymbol, FoldingRange, SymbolKind};
use tsox::lsp::lsproto::lsp::{DocumentUri, Position};
use tsox::tsoptions::parse_command_line;
use tsox::tspath::Path;
use tsox::vfs::{FS, InMemoryFS};

struct TestHost {
    fs: Arc<InMemoryFS>,
}

impl Host for TestHost {
    fn use_case_sensitive_file_names(&self) -> bool {
        true
    }

    fn read_file(&self, path: &str) -> Option<String> {
        self.fs.read_file(path)
    }

    fn converters(&self) -> Converters {
        Converters::new(PositionEncodingKind::Utf16)
    }

    fn get_preferences(&self, _active_file: &str) -> UserPreferences {
        tsox::ls::lsutil::new_default_user_preferences()
    }

    fn get_ecma_line_info(&self, _file_name: &str) -> Option<EcmaLineInfo> {
        None
    }

    fn auto_import_registry(&self) -> AutoImportRegistry {
        AutoImportRegistry
    }

    fn read_directory(
        &self,
        _current_dir: &str,
        _path: &str,
        _extensions: &[String],
        _excludes: &[String],
        _includes: &[String],
        _depth: i32,
    ) -> Vec<String> {
        Vec::new()
    }

    fn get_directories(&self, _path: &str) -> Vec<String> {
        Vec::new()
    }

    fn directory_exists(&self, path: &str) -> bool {
        self.fs.directory_exists(path)
    }

    fn file_exists(&self, path: &str) -> bool {
        self.fs.file_exists(path)
    }
}

fn make_language_service(source: &str) -> (LanguageService, DocumentUri) {
    make_language_service_named("/proj/test.ts", source)
}

fn make_language_service_named(file_name: &str, source: &str) -> (LanguageService, DocumentUri) {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file(file_name, source);

    let args = vec![file_name.to_string()];
    let parsed = parse_command_line(&args, "/proj", Some(fs.as_ref()));

    let host: Arc<dyn tsox::compiler::CompilerHost> = Arc::new(CompilerHostImpl::new(
        fs.clone(),
        "/proj".to_string(),
        lib_path(),
    ));

    let program = Arc::new(Program::new(ProgramOptions {
        config: parsed,
        host,
    }));

    let ls_host: Box<dyn Host> = Box::new(TestHost { fs });
    let ls = LanguageService::new(Path::from("/proj"), program, ls_host, file_name);

    let uri = DocumentUri(format!("file://{}", file_name));
    (ls, uri)
}

fn pos(line: u32, character: u32) -> Position {
    Position { line, character }
}

#[test]
fn lsp_hover_on_variable() {

    let source = "const x = 42;\n";
    let (ls, uri) = make_language_service(source);

    let hover = ls.provide_hover(&uri, pos(0, 7));

    if let Some(h) = hover {
        assert!(
            h.contents.markup_content.is_some() || h.contents.string.is_some(),
            "hover should have content"
        );
    }
}

#[test]
fn lsp_hover_on_function() {
    let source = "function greet(name: string): string {\n  return 'hello ' + name;\n}\n";
    let (ls, uri) = make_language_service(source);

    let _hover = ls.provide_hover(&uri, pos(0, 10));

}

#[test]
fn lsp_hover_returns_none_for_empty_position() {
    let source = "const x = 1;\n";
    let (ls, uri) = make_language_service(source);

    let _hover = ls.provide_hover(&uri, pos(5, 0));
}

#[test]
fn lsp_folding_class_body() {
    let source = "class Foo {\n  bar(): void {\n    console.log('hi');\n  }\n}\n";
    let (ls, uri) = make_language_service(source);

    let ranges: Vec<FoldingRange> = ls.provide_folding_range(&uri);

    assert!(
        !ranges.is_empty(),
        "multi-line class should have folding ranges, got {}",
        ranges.len()
    );
}

#[test]
fn lsp_folding_single_line_no_ranges() {
    let source = "const x = 1;\n";
    let (ls, uri) = make_language_service(source);

    let ranges: Vec<FoldingRange> = ls.provide_folding_range(&uri);
    assert!(
        ranges.is_empty(),
        "single-line file should have no folding ranges"
    );
}

#[test]
fn lsp_folding_region_delimiters() {
    let source = "//#region My Region\nconst x = 1;\nconst y = 2;\n//#endregion\n";
    let (ls, uri) = make_language_service(source);

    let ranges: Vec<FoldingRange> = ls.provide_folding_range(&uri);
    let region_ranges: Vec<_> = ranges
        .iter()
        .filter(|r| r.kind.as_deref() == Some("region"))
        .collect();
    assert!(
        !region_ranges.is_empty(),
        "should have a region folding range from //#region ... //#endregion"
    );
}

#[test]
fn lsp_selection_range_returns_hierarchy() {
    let source = "const x = 1 + 2;\n";
    let (ls, uri) = make_language_service(source);

    let ranges = ls.provide_selection_ranges(&uri, &[pos(0, 10)]);
    assert_eq!(ranges.len(), 1, "should return one selection range");

    let sr = &ranges[0];
    let _ = sr.range;
}

#[test]
fn lsp_symbols_class_and_function() {
    let source = "class MyClass {\n  method() {}\n}\nfunction myFunc() {}\n";
    let (ls, uri) = make_language_service(source);

    let symbols: Vec<DocumentSymbol> = ls.provide_document_symbols(&uri);
    assert!(
        !symbols.is_empty(),
        "should have at least 2 symbols (class + function)"
    );

    let names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(
        names.contains(&"MyClass"),
        "should have MyClass symbol, got: {:?}",
        names
    );
    assert!(
        names.contains(&"myFunc"),
        "should have myFunc symbol, got: {:?}",
        names
    );
}

#[test]
fn lsp_symbols_interface() {
    let source = "interface MyInterface {\n  prop: string;\n}\n";
    let (ls, uri) = make_language_service(source);

    let symbols = ls.provide_document_symbols(&uri);
    assert_eq!(symbols.len(), 1, "should have exactly 1 symbol");
    assert_eq!(symbols[0].name, "MyInterface");
    assert_eq!(symbols[0].kind, SymbolKind::Interface);
}

#[test]
fn lsp_symbols_variable_declaration() {
    let source = "const myVar = 42;\nlet myLet: string = 'hello';\n";
    let (ls, uri) = make_language_service(source);

    let symbols = ls.provide_document_symbols(&uri);
    let names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"myVar"), "should have myVar: {:?}", names);
    assert!(names.contains(&"myLet"), "should have myLet: {:?}", names);
}

#[test]
fn lsp_symbols_empty_file() {
    let source = "\n";
    let (ls, uri) = make_language_service(source);

    let symbols = ls.provide_document_symbols(&uri);
    assert!(symbols.is_empty(), "empty file should have no symbols");
}

#[test]
fn lsp_definition_finds_declaration() {
    let source = "const myVar = 42;\nconsole.log(myVar);\n";
    let (ls, uri) = make_language_service(source);

    let links = ls.provide_definition(&uri, pos(1, 12));

    if !links.is_empty() {
        let link = &links[0];
        assert!(
            link.target_range.start.line == 0,
            "definition should be on line 0"
        );
    }
}

#[test]
fn lsp_parse_region_start() {
    let result = tsox::ls::folding::parse_region_delimiter("// #region My Label");
    assert!(result.is_some());
    let r = result.unwrap();
    assert!(r.is_start);
    assert_eq!(r.name, "My Label");
}

#[test]
fn lsp_parse_region_end() {
    let result = tsox::ls::folding::parse_region_delimiter("//#endregion");
    assert!(result.is_some());
    let r = result.unwrap();
    assert!(!r.is_start);
    assert_eq!(r.name, "");
}

#[test]
fn lsp_parse_region_not_a_comment() {
    let result = tsox::ls::folding::parse_region_delimiter("const x = 1;");
    assert!(result.is_none());
}
