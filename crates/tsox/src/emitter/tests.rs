use super::*;
use crate::core::tristate::Tristate;
use crate::parser::Parser;
use crate::vfs::InMemoryFS;

fn parse(source: &str) -> SourceFile {
    let (file, _diags) =
        Parser::parse_source_file_text_with_diagnostics("/test.ts", source.to_string());
    file
}

fn emit_to_string(source: &str) -> String {
    let sf = parse(source);
    emit_js_text(&sf, &CompilerOptions::default())
}

fn emit_to_string_no_comments(source: &str) -> String {
    let sf = parse(source);
    let mut opts = CompilerOptions::default();
    opts.remove_comments = Tristate::True;
    emit_js_text(&sf, &opts)
}

fn emit_to_string_es5(source: &str) -> String {
    let sf = parse(source);
    let mut opts = CompilerOptions::default();
    opts.target = ScriptTarget::ES5;
    emit_js_text(&sf, &opts)
}

#[test]
fn emit_strips_interface_declaration() {
    let js = emit_to_string("interface Foo { x: number; }\nlet y = 1;");
    assert!(!js.contains("interface"));
    assert!(js.contains("let y = 1;"));
}

#[test]
fn emit_strips_type_alias() {
    let js = emit_to_string("type MyType = number;\nlet y: MyType = 1;");
    assert!(!js.contains("type MyType"));
    assert!(js.contains("let y = 1;"));
}

#[test]
fn emit_strips_variable_type_annotation() {
    let js = emit_to_string("let x: number = 1;");
    assert_eq!(js.trim(), "let x = 1;");
}

#[test]
fn emit_strips_multiple_variable_declarations() {
    let js = emit_to_string("let a: number = 1;\nlet b: string = \"hello\";");
    assert!(js.contains("let a = 1;"));
    assert!(js.contains("let b = \"hello\";"));
    assert!(!js.contains(": number"));
    assert!(!js.contains(": string"));
}

#[test]
fn emit_strips_function_param_types() {
    let js = emit_to_string("function foo(a: number, b: string): number { return a; }");
    assert!(js.contains("function foo(a, b)"));
    assert!(!js.contains(": number"));
    assert!(js.contains("return a;"));
}

#[test]
fn emit_strips_function_return_type() {
    let js = emit_to_string("function bar(): void { console.log(\"hi\"); }");
    assert!(js.contains("function bar()"));
    assert!(!js.contains(": void"));
}

#[test]
fn emit_strips_type_parameters() {
    let js = emit_to_string("function identity<T>(x: T): T { return x; }");
    assert!(js.contains("function identity(x)"));
    assert!(!js.contains("<T>"));
}

#[test]
fn emit_strips_as_expression() {
    let js = emit_to_string("function f(x: number) { return x; }");
    assert!(!js.contains(": number"));
    assert!(js.contains("return x;"));
}

#[test]
fn emit_preserves_expression_statement() {
    let js = emit_to_string("console.log(\"hello\");");
    assert_eq!(js.trim(), "console.log(\"hello\");");
}

#[test]
fn emit_preserves_class() {
    let js = emit_to_string(
        "class Foo { x: number = 1; method(a: string): void { this.x = a.length; } }",
    );
    assert!(js.contains("class Foo"));
    assert!(js.contains("x = 1"));
    assert!(js.contains("method(a)"));
    assert!(!js.contains(": void"));
}

#[test]
fn emit_strips_property_type_annotation() {
    let js = emit_to_string("class Bar { prop: string = \"hi\"; }");
    assert!(js.contains("prop = \"hi\""));
    assert!(!js.contains(": string"));
}

#[test]
fn emit_preserves_if_statement() {
    let js = emit_to_string("if (x > 0) { console.log(x); } else { console.log(0); }");
    assert!(js.contains("if (x > 0)"));
    assert!(js.contains("else"));
}

#[test]
fn emit_preserves_for_loop() {
    let js = emit_to_string("for (let i: number = 0; i < 10; i++) { console.log(i); }");
    assert!(js.contains("for (let i = 0; i < 10; i++)"));
}

#[test]
fn emit_preserves_arrow_function() {
    let js = emit_to_string("function fn(x: number): number { return x * 2; }");
    assert!(js.contains("function fn(x)"));
    assert!(js.contains("return x * 2;"));
    assert!(!js.contains(": number"));
}

#[test]
fn emit_writes_file_to_fs() {
    let fs = InMemoryFS::new();
    fs.insert_file("/test.ts", "let x: number = 1;");

    let sf = parse("let x: number = 1;");
    let mut sf_with_name = sf;
    sf_with_name.file_name = "/test.ts".to_string();

    let options = CompilerOptions::default();
    let result = emit_source_file(&sf_with_name, &options, &fs, &|path, data| {
        fs.write_file(path, data)
    });

    assert_eq!(result.emitted_files, vec!["/test.js".to_string()]);
    assert_eq!(fs.read_file("/test.js").unwrap(), "let x = 1;");
}

#[test]
fn emit_respects_out_dir() {
    let fs = InMemoryFS::new();
    let sf = parse("let x = 1;");
    let mut sf_with_name = sf;
    sf_with_name.file_name = "/src/test.ts".to_string();

    let mut options = CompilerOptions::default();
    options.out_dir = "/dist".to_string();
    let result = emit_source_file(&sf_with_name, &options, &fs, &|path, data| {
        fs.write_file(path, data)
    });

    assert_eq!(result.emitted_files, vec!["/dist/test.js".to_string()]);
}

#[test]
fn emit_skips_json_files() {
    let sf = parse("{}");
    let mut sf_with_name = sf;
    sf_with_name.file_name = "/test.json".to_string();
    sf_with_name.script_kind = crate::ast::ScriptKind::Json;

    let options = CompilerOptions::default();
    let fs = InMemoryFS::new();
    let result = emit_source_file(&sf_with_name, &options, &fs, &|path, data| {
        fs.write_file(path, data)
    });

    assert!(result.emitted_files.is_empty());
}

#[test]
fn emit_output_extension_mjs() {
    assert_eq!(get_output_extension("/test.mts"), ".mjs");
    assert_eq!(get_output_extension("/test.cts"), ".cjs");
    assert_eq!(get_output_extension("/test.ts"), ".js");
    assert_eq!(get_output_extension("/test.json"), ".json");
    assert_eq!(get_output_extension("/test.tsx"), ".js");
    assert_eq!(get_output_extension("/test.jsx"), ".js");
    assert_eq!(get_output_extension("/test.mjs"), ".mjs");
    assert_eq!(get_output_extension("/test.cjs"), ".cjs");
}

#[test]
fn emit_program_emits_multiple_files() {
    let fs = InMemoryFS::new();

    let sf1 = parse("let x: number = 1;");
    let mut sf1 = sf1;
    sf1.file_name = "/a.ts".to_string();

    let sf2 = parse("function foo(a: string): void { console.log(a); }");
    let mut sf2 = sf2;
    sf2.file_name = "/b.ts".to_string();

    let source_files = vec![Arc::new(sf1), Arc::new(sf2)];
    let options = CompilerOptions::default();
    let result = emit_program(&source_files, &options, &fs, &|path, data| {
        fs.write_file(path, data)
    });

    assert_eq!(result.emitted_files.len(), 2);
    assert!(result.emitted_files.contains(&"/a.js".to_string()));
    assert!(result.emitted_files.contains(&"/b.js".to_string()));
    assert!(!result.emit_skipped);
    assert!(fs.file_exists("/a.js"));
    assert!(fs.file_exists("/b.js"));
    assert_eq!(fs.read_file("/a.js").unwrap().trim(), "let x = 1;");
    assert!(fs.read_file("/b.js").unwrap().contains("function foo(a)"));
}

#[test]
fn emit_program_aggregates_diagnostics_on_write_failure() {
    let sf = parse("let x: number = 1;");
    let mut sf = sf;
    sf.file_name = "/test.ts".to_string();

    let source_files = vec![Arc::new(sf)];
    let options = CompilerOptions::default();
    let fs = InMemoryFS::new();
    let result = emit_program(&source_files, &options, &fs, &|_path, _data| {
        Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "denied",
        ))
    });

    assert!(result.emitted_files.is_empty());
    assert!(result.emit_skipped);
    assert!(!result.diagnostics.is_empty());
    assert!(result.diagnostics[0].contains("Error writing"));
}

#[test]
fn emit_program_handles_empty_source() {
    let fs = InMemoryFS::new();
    let sf = parse("");
    let mut sf = sf;
    sf.file_name = "/empty.ts".to_string();

    let source_files = vec![Arc::new(sf)];
    let options = CompilerOptions::default();
    let result = emit_program(&source_files, &options, &fs, &|path, data| {
        fs.write_file(path, data)
    });

    assert_eq!(result.emitted_files, vec!["/empty.js".to_string()]);
    assert!(fs.file_exists("/empty.js"));
}

#[test]
fn emit_source_file_skips_js_when_no_emit_for_js_files() {
    let fs = InMemoryFS::new();
    let sf = parse("let x = 1;");
    let mut sf = sf;
    sf.file_name = "/test.js".to_string();
    sf.script_kind = crate::ast::ScriptKind::Js;

    let mut options = CompilerOptions::default();
    options.no_emit_for_js_files = Tristate::True;
    let result = emit_source_file(&sf, &options, &fs, &|path, data| fs.write_file(path, data));

    assert!(result.emitted_files.is_empty());
    assert!(!fs.file_exists("/test.js"));
}

#[test]
fn emit_source_file_emits_js_by_default() {
    let fs = InMemoryFS::new();
    let sf = parse("let x = 1;");
    let mut sf = sf;
    sf.file_name = "/test.js".to_string();
    sf.script_kind = crate::ast::ScriptKind::Js;

    let options = CompilerOptions::default();
    let result = emit_source_file(&sf, &options, &fs, &|path, data| fs.write_file(path, data));

    assert_eq!(result.emitted_files, vec!["/test.js".to_string()]);
}

#[test]
fn emit_preserves_export_declaration() {
    let js = emit_to_string("export const x = 1;");
    assert!(js.contains("export const x = 1;"));
}

#[test]
fn emit_preserves_import_declaration() {
    let js = emit_to_string("import { foo } from \"./bar\";");
    assert!(js.contains("import { foo }"));
    assert!(js.contains("from \"./bar\";"));
}

#[test]
fn emit_strips_export_from_type_alias() {
    let js = emit_to_string("export type ID = number;\nlet x: ID = 1;");
    assert!(!js.contains("type ID"));
    assert!(js.contains("let x = 1;"));
}

#[test]
fn emit_preserves_class_with_constructor() {
    let js = emit_to_string("class Foo { x: number; constructor(x: number) { this.x = x; } }");
    assert!(js.contains("class Foo"));
    assert!(js.contains("constructor(x)"));
    assert!(js.contains("this.x = x;"));
    assert!(!js.contains(": number"));
}

#[test]
fn emit_preserves_while_loop() {
    let js = emit_to_string("let i: number = 0; while (i < 10) { i++; }");
    assert!(js.contains("let i = 0;"));
    assert!(js.contains("while (i < 10)"));
    assert!(js.contains("i++;"));
}

#[test]
fn emit_preserves_do_while_loop() {
    let js = emit_to_string("let i: number = 0; do { i++; } while (i < 10);");
    assert!(js.contains("let i = 0;"));
    assert!(js.contains("do {"));
    assert!(js.contains("} while (i < 10);"));
}

#[test]
fn emit_strips_function_type_parameters_in_class_method() {
    let js = emit_to_string("class Foo { method(x: number): number { return x; } }");
    assert!(js.contains("method(x)"));
    assert!(!js.contains(": number"));
    assert!(js.contains("return x;"));
}

#[test]
fn remove_comments_strips_single_line_comment() {
    let js = emit_to_string_no_comments("// This comment should be removed\nconst e = 5;");
    assert!(!js.contains("// This comment"));
    assert!(js.contains("const e = 5;"));
}

#[test]
fn remove_comments_strips_multi_line_comment() {
    let js = emit_to_string_no_comments("/* block comment */ const x = 1;");
    assert!(!js.contains("block comment"));
    assert!(js.contains("const x = 1;"));
}

#[test]
fn remove_comments_strips_jsdoc_comment() {
    let js = emit_to_string_no_comments("/** JSDoc */\nfunction foo() { return 1; }");
    assert!(!js.contains("JSDoc"));
    assert!(js.contains("function foo()"));
}

#[test]
fn remove_comments_preserves_comments_in_strings() {
    let js = emit_to_string_no_comments("// real comment\nconst s = \"// not a comment\";");
    assert!(!js.contains("real comment"));
    assert!(js.contains("\"// not a comment\""));
}

#[test]
fn remove_comments_preserves_comments_in_template_literals() {
    let js = emit_to_string_no_comments("// real comment\nconst s = `// not a comment ${1}`;");
    assert!(!js.contains("real comment"));
    assert!(js.contains("`// not a comment"));
}

#[test]
fn remove_comments_does_not_affect_division() {
    let js = emit_to_string_no_comments("const x = 10 / 2;");
    assert!(js.contains("10 / 2"));
}

#[test]
fn remove_comments_strips_trailing_comment() {
    let js = emit_to_string_no_comments("const x = 1; // trailing");
    assert!(js.contains("const x = 1;"));
    assert!(!js.contains("trailing"));
}

#[test]
fn remove_comments_off_by_default() {
    let js = emit_to_string("// comment\nconst x = 1;");
    assert!(js.contains("// comment"));
}

#[test]
fn es5_downlevels_const_to_var() {
    let js = emit_to_string_es5("const f: number = 6;");
    assert!(js.contains("var f = 6;"));
    assert!(!js.contains("const"));
    assert!(!js.contains(": number"));
}

#[test]
fn es5_downlevels_let_to_var() {
    let js = emit_to_string_es5("let x: number = 1;");
    assert!(js.contains("var x = 1;"));
    assert!(!js.contains("let"));
}

#[test]
fn es5_downlevels_const_with_export() {
    let js = emit_to_string_es5("const f: number = 6;\nexport { f };");
    assert!(js.contains("var f = 6;"));
    assert!(js.contains("export { f };"));
}

#[test]
fn es5_preserves_var() {
    let js = emit_to_string_es5("var x = 1;");
    assert!(js.contains("var x = 1;"));
}

#[test]
fn es5_downlevels_nested_let_in_for_loop() {
    let js = emit_to_string_es5("for (let i = 0; i < 10; i++) { console.log(i); }");
    assert!(js.contains("for (var i = 0;"));
}

#[test]
fn es5_no_downlevel_when_target_es2015() {
    let sf = parse("const x = 1;");
    let mut opts = CompilerOptions::default();
    opts.target = ScriptTarget::ES2015;
    let js = emit_js_text(&sf, &opts);
    assert!(js.contains("const x = 1;"));
    assert!(!js.contains("var x"));
}

fn emit_to_string_commonjs(source: &str) -> String {
    let sf = parse(source);
    let mut opts = CompilerOptions::default();
    opts.module = ModuleKind::CommonJS;
    emit_js_text(&sf, &opts)
}

#[test]
fn commonjs_starts_with_use_strict() {
    let js = emit_to_string_commonjs("const x = 1;");
    assert!(js.starts_with("\"use strict\";\n"));
}

#[test]
fn commonjs_export_named() {
    let js = emit_to_string_commonjs("const x = 1;\nexport { x };");
    assert!(js.contains("const x = 1;"));
    assert!(js.contains("exports.x = x;"));
    assert!(!js.contains("export { x }"));
}

#[test]
fn commonjs_export_default_expression() {
    let js = emit_to_string_commonjs("export default 42;");
    assert!(js.contains("exports.default = 42;"));
    assert!(!js.contains("export default"));
}

#[test]
fn commonjs_export_equals() {
    let js = emit_to_string_commonjs("const obj = {};\nexport = obj;");
    assert!(js.contains("module.exports = obj;"));
}

#[test]
fn commonjs_export_const() {
    let js = emit_to_string_commonjs("export const x = 1;");
    assert!(js.contains("const x = 1;"));
    assert!(js.contains("exports.x = x;"));
    assert!(!js.contains("export const"));
}

#[test]
fn commonjs_export_function() {
    let js = emit_to_string_commonjs("export function foo() { return 1; }");
    assert!(js.contains("function foo()"));
    assert!(js.contains("exports.foo = foo;"));
    assert!(!js.contains("export function"));
}

#[test]
fn commonjs_export_class() {
    let js = emit_to_string_commonjs("export class Foo { }");
    assert!(js.contains("class Foo"));
    assert!(js.contains("exports.Foo = Foo;"));
    assert!(!js.contains("export class"));
}

#[test]
fn commonjs_import_named() {
    let js = emit_to_string_commonjs("import { foo } from \"./bar\";");
    assert!(js.contains("const { foo } = require(\"./bar\");"));
    assert!(!js.contains("import { foo }"));
}

#[test]
fn commonjs_import_namespace() {
    let js = emit_to_string_commonjs("import * as ns from \"./bar\";");
    assert!(js.contains("const ns = require(\"./bar\");"));
}

#[test]
fn commonjs_import_default() {
    let js = emit_to_string_commonjs("import d from \"./bar\";");
    assert!(js.contains("const { default: d } = require(\"./bar\");"));
}

#[test]
fn commonjs_import_side_effect() {
    let js = emit_to_string_commonjs("import \"./bar\";");
    assert!(js.contains("require(\"./bar\");"));
}

#[test]
fn commonjs_import_type_stripped() {
    let js = emit_to_string_commonjs("import type { foo } from \"./bar\";");
    assert!(!js.contains("import"));
    assert!(!js.contains("require"));
}

#[test]
fn commonjs_export_multiple_named() {
    let js = emit_to_string_commonjs("const x = 1;\nconst y = 2;\nexport { x, y };");
    assert!(js.contains("exports.x = x;"));
    assert!(js.contains("exports.y = y;"));
}

#[test]
fn commonjs_export_reexport() {
    let js = emit_to_string_commonjs("export { foo } from \"./bar\";");
    assert!(js.contains("const { foo } = require(\"./bar\");"));
    assert!(js.contains("exports.foo = foo;"));
}

fn emit_with_sourcemap(
    source: &str,
    source_map: bool,
    inline: bool,
    inline_sources: bool,
) -> (String, Option<String>, String) {
    let sf = parse(source);
    let mut opts = CompilerOptions::default();
    if source_map {
        opts.source_map = Tristate::True;
    }
    if inline {
        opts.inline_source_map = Tristate::True;
    }
    if inline_sources {
        opts.inline_sources = Tristate::True;
    }
    emit_js_with_sourcemap(&sf, &opts, "/test.js")
}

#[test]
fn sourcemap_produces_valid_json() {
    let (js, map_json, url) = emit_with_sourcemap("let x = 1;\nlet y = 2;\n", true, false, false);

    assert!(!js.contains("sourceMappingURL"));

    let map = map_json.expect("map_json should be Some");
    assert!(map.contains("\"version\":3"));
    assert!(map.contains("\"file\":\"test.js\""));
    assert!(map.contains("\"sources\""));
    assert!(map.contains("\"mappings\""));

    assert!(!map.contains("\"mappings\":\"\""));

    assert_eq!(url, "test.js.map");

    assert!(!map.contains("sourcesContent"));
}

#[test]
fn sourcemap_inline_produces_data_url() {
    let (js, map_json, url) = emit_with_sourcemap("let x = 1;\n", false, true, false);

    assert!(map_json.is_none());

    assert!(url.starts_with("data:application/json;base64,"));

    assert!(!js.contains("sourceMappingURL"));
}

#[test]
fn sourcemap_inline_sources_includes_content() {
    let (_js, map_json, _url) = emit_with_sourcemap("let x = 1;\n", true, false, true);
    let map = map_json.expect("map_json should be Some");
    assert!(map.contains("sourcesContent"));
    assert!(map.contains("let x = 1;"));
}

#[test]
fn sourcemap_strips_type_annotations() {
    let (js, map_json, _url) = emit_with_sourcemap("let x: number = 1;\n", true, false, false);

    assert!(js.contains("let x = 1;"));
    assert!(!js.contains(": number"));

    let map = map_json.expect("map_json should be Some");
    assert!(map.contains("\"version\":3"));
}

#[test]
fn sourcemap_mappings_decode_to_correct_positions() {
    use crate::sourcemap::MappingsDecoder;
    let (js, map_json, _url) = emit_with_sourcemap("let x = 1;\n", true, false, false);
    let map = map_json.expect("map_json should be Some");

    let raw: crate::sourcemap::RawSourceMap = crate::json::unmarshal(&map).expect("valid JSON");
    assert_eq!(raw.version, 3);
    assert!(!raw.sources.is_empty());
    assert!(!raw.mappings.is_empty());

    let mut decoder = MappingsDecoder::new(&raw.mappings);
    let mut count = 0;
    let mut has_source_mapping = false;
    while count < 100 {
        match decoder.next() {
            Some(m) => {
                if m.is_source_mapping() {
                    has_source_mapping = true;

                    assert!(m.source_line >= 0);
                }
                count += 1;
            }
            None => break,
        }
    }
    assert!(
        has_source_mapping,
        "should have at least one source mapping"
    );

    let gen_lines = js.lines().count();
    let _ = gen_lines;
}

#[test]
fn sourcemap_not_emitted_by_default() {
    let sf = parse("let x = 1;\n");
    let js = emit_js_text(&sf, &CompilerOptions::default());
    assert!(!js.contains("sourceMappingURL"));
}

#[test]
fn sourcemap_commonjs_use_strict_not_mapped() {
    let mut opts = CompilerOptions::default();
    opts.source_map = Tristate::True;
    opts.module = ModuleKind::CommonJS;
    let sf = parse("const x = 1;\nexport { x };");
    let (js, map_json, _url) = emit_js_with_sourcemap(&sf, &opts, "/test.js");
    assert!(js.starts_with("\"use strict\";"));
    let map = map_json.expect("map_json should be Some");
    assert!(map.contains("\"version\":3"));
}

#[test]
fn sourcemap_write_file_creates_map() {
    use std::cell::RefCell;
    let sf = parse("let x = 1;\nlet y: string = \"hi\";\n");
    let mut opts = CompilerOptions::default();
    opts.source_map = Tristate::True;
    let written: RefCell<Vec<(String, String)>> = RefCell::new(Vec::new());
    let result = emit_source_file_with_common_dir(
        &sf,
        &opts,
        &crate::vfs::InMemoryFS::new(),
        "",
        &|path, content| {
            written
                .borrow_mut()
                .push((path.to_string(), content.to_string()));
            Ok(())
        },
    );

    assert_eq!(result.emitted_files.len(), 2);
    let written = written.borrow();
    let js_file = &written
        .iter()
        .find(|(p, _)| p.ends_with(".js") && !p.ends_with(".map"))
        .expect("js file")
        .1;
    let map_file = &written
        .iter()
        .find(|(p, _)| p.ends_with(".js.map"))
        .expect("map file")
        .1;

    assert!(js_file.contains("//# sourceMappingURL="));

    assert!(map_file.contains("\"version\":3"));
    assert!(map_file.contains("\"mappings\""));

    assert!(js_file.contains("let y = \"hi\";"));
    assert!(!js_file.contains(": string"));
}

fn emit_dts(source: &str) -> String {
    let sf = parse(source);
    let opts = CompilerOptions::default();
    emit_declaration_text(&sf, &opts)
}

#[test]
fn dts_function_strips_body() {
    let dts = emit_dts("function add(a: number, b: number): number { return a + b; }");
    assert!(dts.contains("declare function add(a: number, b: number): number;"));
    assert!(!dts.contains("return"));
    assert!(!dts.contains("{"));
}

#[test]
fn dts_export_function_adds_declare() {
    let dts = emit_dts("export function foo(): void { console.log(1); }");
    assert!(dts.contains("export declare function foo(): void;"));
    assert!(!dts.contains("console"));
}

#[test]
fn dts_variable_strips_initializer() {
    let dts = emit_dts("const x: number = 42;");
    assert!(dts.contains("declare const x: number;"));
    assert!(!dts.contains("42"));
}

#[test]
fn dts_export_variable_strips_initializer() {
    let dts = emit_dts("export const PI: number = 3.14;");
    assert!(dts.contains("export declare const PI: number;"));
    assert!(!dts.contains("3.14"));
}

#[test]
fn dts_interface_emitted_as_is() {
    let dts = emit_dts("interface User { id: number; name: string; }");
    assert!(dts.contains("interface User {"));
    assert!(dts.contains("id: number;"));
    assert!(!dts.contains("declare"));
}

#[test]
fn dts_type_alias_emitted_as_is() {
    let dts = emit_dts("type ID = string | number;");
    assert!(dts.contains("type ID = string | number;"));
    assert!(!dts.contains("declare"));
}

#[test]
fn dts_enum_adds_declare() {
    let dts = emit_dts("enum Color { Red, Green, Blue }");
    assert!(dts.contains("declare enum Color {"));
    assert!(dts.contains("Red"));
}

#[test]
fn dts_runtime_statements_skipped() {
    let dts = emit_dts("console.log(\"hello\");\nlet x: number = 1;");
    assert!(!dts.contains("console"));
    assert!(!dts.contains("hello"));
    assert!(dts.contains("declare let x: number;"));
}

#[test]
fn dts_multiple_declarations() {
    let src = "export function add(a: number, b: number): number { return a + b; }\n\
                   export const PI: number = 3.14;\n\
                   export interface User { id: number; }\n\
                   export type ID = string | number;\n";
    let dts = emit_dts(src);
    assert!(dts.contains("export declare function add(a: number, b: number): number;"));
    assert!(dts.contains("export declare const PI: number;"));
    assert!(dts.contains("export interface User {"));
    assert!(dts.contains("export type ID = string | number;"));
    assert!(!dts.contains("return"));
    assert!(!dts.contains("3.14"));
}

#[test]
fn dts_class_adds_declare() {
    let dts = emit_dts("export class Point { x: number; constructor(x: number) { this.x = x; } }");
    assert!(dts.contains("export declare class Point"));
}

#[test]
fn dts_write_file_creates_dts() {
    use std::cell::RefCell;
    let sf = parse("export function foo(): number { return 1; }\nexport const x: number = 42;\n");
    let mut opts = CompilerOptions::default();
    opts.declaration = Tristate::True;
    let written: RefCell<Vec<(String, String)>> = RefCell::new(Vec::new());
    let result = emit_source_file_with_common_dir(
        &sf,
        &opts,
        &crate::vfs::InMemoryFS::new(),
        "",
        &|path, content| {
            written
                .borrow_mut()
                .push((path.to_string(), content.to_string()));
            Ok(())
        },
    );

    assert!(result.emitted_files.iter().any(|p| p.ends_with(".js")));
    assert!(result.emitted_files.iter().any(|p| p.ends_with(".d.ts")));
    let written = written.borrow();
    let dts_file = &written
        .iter()
        .find(|(p, _)| p.ends_with(".d.ts"))
        .expect("dts file")
        .1;

    assert!(dts_file.contains("export declare function foo(): number;"));
    assert!(dts_file.contains("export declare const x: number;"));
    assert!(!dts_file.contains("return"));
    assert!(!dts_file.contains("42"));
}

#[test]
fn dts_emit_declaration_only_suppresses_js() {
    use std::cell::RefCell;
    let sf = parse("export function foo(): number { return 1; }\n");
    let mut opts = CompilerOptions::default();
    opts.declaration = Tristate::True;
    opts.emit_declaration_only = Tristate::True;
    let written: RefCell<Vec<(String, String)>> = RefCell::new(Vec::new());
    let result = emit_source_file_with_common_dir(
        &sf,
        &opts,
        &crate::vfs::InMemoryFS::new(),
        "",
        &|path, content| {
            written
                .borrow_mut()
                .push((path.to_string(), content.to_string()));
            Ok(())
        },
    );

    assert!(!result.emitted_files.iter().any(|p| p.ends_with(".js")));
    assert!(result.emitted_files.iter().any(|p| p.ends_with(".d.ts")));
}

#[test]
fn dts_drops_value_imports_keeps_side_effect() {
    let src = "import { useState } from 'react';\n\
                   import reactLogo from './assets/react.svg';\n\
                   import './App.css';\n\
                   export default function App() { return 1; }\n";
    let dts = emit_dts(src);

    assert!(!dts.contains("useState"));
    assert!(!dts.contains("reactLogo"));

    assert!(dts.contains("import './App.css';"));

    assert!(dts.contains("export default function App(): unknown;"));
    assert!(!dts.contains("return"));
}

#[test]
fn dts_keeps_type_only_import() {
    let src = "import type { Config } from './config';\n\
                   import { value } from './values';\n\
                   export const c: Config = {} as any;\n";
    let dts = emit_dts(src);

    assert!(dts.contains("import type { Config } from './config';"));

    assert!(!dts.contains("value"));
}

#[test]
fn dts_function_declare_keyword_and_semicolon() {
    let dts = emit_dts("function add(a: number, b: number): number { return a + b; }");
    assert!(dts.contains("declare function add(a: number, b: number): number;"));
}

#[test]
fn dts_class_strips_method_bodies() {
    let src = "export class Counter {\n\
                   count: number;\n\
                   constructor(initial: number) { this.count = initial; }\n\
                   increment(): void { this.count++; }\n\
                   }\n";
    let dts = emit_dts(src);
    assert!(dts.contains("export declare class Counter"));

    assert!(dts.contains("count: number;"));

    assert!(dts.contains("constructor(initial: number);"));
    assert!(dts.contains("increment(): void;"));

    assert!(!dts.contains("this.count"));
    assert!(!dts.contains("initial;"));
}

#[test]
fn dts_variable_strips_initializer_without_type() {
    let dts = emit_dts("const answer = 42;");
    assert!(dts.contains("declare const answer;"));
    assert!(!dts.contains("42"));
}

#[test]
fn dts_variable_multiple_no_type() {
    let dts = emit_dts("let a = 1;\nlet b = 2;");
    assert!(dts.contains("declare let a;"));
    assert!(dts.contains("declare let b;"));
    assert!(!dts.contains("= 1"));
    assert!(!dts.contains("= 2"));
}

#[test]
fn type_eraser() {
    let cases: &[(&str, &[&str], &[&str])] = &[
        ("interface I { x: number; }", &[], &["interface"]),
        ("type T = number;", &[], &["type T"]),
        (
            "function f<T>(x: T): T { return x; }",
            &["function f(x)", "return x;"],
            &["<T>", ": T"],
        ),
        (
            "function add(a: number, b: string): void { return a; }",
            &["function add(a, b)", "return a;"],
            &[": number", ": string", ": void"],
        ),
        ("let x: number = 1;", &["let x = 1;"], &[": number"]),
        (
            "const s: string = \"hi\";",
            &["const s = \"hi\";"],
            &[": string"],
        ),
        (
            "class C { x: number = 1; m(a: string): void { return a; } }",
            &["class C", "x = 1", "m(a)", "return a;"],
            &[": number", ": string", ": void"],
        ),
    ];

    for &(input, must_contain, must_not_contain) in cases {
        let js = emit_to_string(input);
        for needle in must_contain {
            assert!(
                js.contains(needle),
                "type_eraser({input:?}): expected output to contain {needle:?}, got {js:?}"
            );
        }
        for needle in must_not_contain {
            assert!(
                !js.contains(needle),
                "type_eraser({input:?}): expected output to NOT contain {needle:?}, got {js:?}"
            );
        }
    }
}

#[test]
fn import_elision() {
    for input in [
        "import type { foo } from \"./bar\";",
        "import type * as ns from \"./bar\";",
        "import type d from \"./bar\";",
    ] {
        let js = emit_to_string_commonjs(input);
        assert!(
            !js.contains("require"),
            "import_elision({input:?}): type-only import should be elided, got {js:?}"
        );
        assert!(
            !js.contains("import"),
            "import_elision({input:?}): type-only import should be elided, got {js:?}"
        );
    }

    let js = emit_to_string_commonjs("import { foo } from \"./bar\";");
    assert!(
        js.contains("require(\"./bar\")"),
        "import_elision: value import should be retained, got {js:?}"
    );

    let js = emit_to_string_commonjs("import { type foo, bar } from \"./bar\";");
    assert!(
        js.contains("bar"),
        "import_elision: value binding 'bar' should be retained, got {js:?}"
    );
    assert!(
        !js.contains("foo"),
        "import_elision: type-only binding 'foo' should be elided, got {js:?}"
    );
    assert!(
        js.contains("require(\"./bar\")"),
        "import_elision: mixed import should still require the module, got {js:?}"
    );
}

fn parse_tsx(source: &str) -> SourceFile {
    let (file, _diags) =
        Parser::parse_source_file_text_with_diagnostics("/test.tsx", source.to_string());
    file
}

fn emit_to_string_jsx(source: &str) -> String {
    let sf = parse_tsx(source);
    let mut opts = CompilerOptions::default();
    opts.jsx = JsxEmit::ReactJSX;
    emit_js_text(&sf, &opts)
}

#[test]
fn jsx_self_closing_element() {
    let js = emit_to_string_jsx("const x = <div />;");
    assert!(js.contains("_jsx(\"div\", {})"));
    assert!(
        js.contains("import { jsx as _jsx } from \"react/jsx-runtime\";"),
        "expected jsx import, got {js:?}"
    );
}

#[test]
fn jsx_element_with_string_attribute() {
    let js = emit_to_string_jsx("const x = <div className=\"x\" />;");
    assert!(js.contains("_jsx(\"div\", { className: \"x\" })"));
}

#[test]
fn jsx_element_with_expression_attribute() {
    let js = emit_to_string_jsx("const x = <div onClick={handler} />;");
    assert!(js.contains("onClick: handler"));
}

#[test]
fn jsx_element_with_boolean_attribute() {
    let js = emit_to_string_jsx("const x = <input disabled />;");
    assert!(js.contains("disabled: true"));
}

#[test]
fn jsx_element_with_single_text_child() {
    let js = emit_to_string_jsx("const x = <h1>Hello</h1>;");
    assert!(js.contains("_jsx(\"h1\", { children: \"Hello\" })"));
}

#[test]
fn jsx_element_with_single_element_child() {
    let js = emit_to_string_jsx("const x = <div><span /></div>;");
    assert!(
        js.contains("children: _jsx(\"span\", {})"),
        "expected single element child, got {js:?}"
    );
}

#[test]
fn jsx_element_with_multiple_children() {
    let js = emit_to_string_jsx("const x = <div><span /><p /></div>;");
    assert!(js.contains("_jsxs(\"div\","));
    assert!(js.contains("children: [_jsx(\"span\", {}), _jsx(\"p\", {})]"));
}

#[test]
fn jsx_fragment() {
    let js = emit_to_string_jsx("const x = <><span /><p /></>;");
    assert!(js.contains("_jsxs(_Fragment,"));
    assert!(
        js.contains("import { Fragment as _Fragment, jsx as _jsx, jsxs as _jsxs }"),
        "expected all three imports, got {js:?}"
    );
}

#[test]
fn jsx_fragment_empty() {
    let js = emit_to_string_jsx("const x = <></>;");
    assert!(js.contains("_jsx(_Fragment, {})"));
}

#[test]
fn jsx_expression_child() {
    let js = emit_to_string_jsx("const x = <div>{count}</div>;");
    assert!(js.contains("children: count"));
}

#[test]
fn jsx_mixed_children() {
    let js = emit_to_string_jsx("const x = <p>Edit <code>file</code> now</p>;");
    assert!(js.contains("_jsxs(\"p\","));
    assert!(js.contains("\"Edit \""));
    assert!(js.contains("_jsx(\"code\", { children: \"file\" })"));
    assert!(js.contains("\" now\""));
}

#[test]
fn jsx_component_tag() {
    let js = emit_to_string_jsx("const x = <Foo bar=\"1\" />;");
    assert!(js.contains("_jsx(Foo, { bar: \"1\" })"));
}

#[test]
fn jsx_member_expression_tag() {
    let js = emit_to_string_jsx("const x = <Foo.Bar />;");
    assert!(js.contains("_jsx(Foo.Bar, {})"));
}

#[test]
fn jsx_namespaced_attribute() {
    let js = emit_to_string_jsx("const x = <div aria-hidden=\"true\" />;");
    assert!(js.contains("\"aria-hidden\": \"true\""));
}

#[test]
fn jsx_import_injection() {
    let js = emit_to_string_jsx("const x = <div />;");
    let import_line = js
        .lines()
        .find(|l| l.starts_with("import"))
        .expect("should have an import");
    assert_eq!(
        import_line,
        "import { jsx as _jsx } from \"react/jsx-runtime\";"
    );
}

#[test]
fn jsx_import_only_used_helpers() {
    let js = emit_to_string_jsx("const x = <div><a /><b /></div>;");
    assert!(!js.contains("Fragment as _Fragment"));
    assert!(js.contains("jsx as _jsx"));
    assert!(js.contains("jsxs as _jsxs"));
}

#[test]
fn jsx_preserves_expression_in_attribute() {
    let js = emit_to_string_jsx("const x = <button onClick={() => fn(1)}>click</button>;");
    assert!(js.contains("onClick: () => fn(1)"));
    assert!(js.contains("children: \"click\""));
}

#[test]
fn jsx_nested_elements() {
    let js = emit_to_string_jsx("const x = <div><span><p /></span></div>;");
    assert!(js.contains("children: _jsx(\"span\", { children: _jsx(\"p\", {}) })"));
}

#[test]
fn jsx_no_transform_when_not_tsx() {
    let (sf, _diags) =
        Parser::parse_source_file_text_with_diagnostics("/test.ts", "const x = 1;".to_string());
    let mut opts = CompilerOptions::default();
    opts.jsx = JsxEmit::ReactJSX;
    let js = emit_js_text(&sf, &opts);
    assert!(!js.contains("_jsx"));
}

#[test]
fn jsx_empty_element_no_children_prop() {
    let js = emit_to_string_jsx("const x = <section id=\"main\"></section>;");
    assert!(js.contains("_jsx(\"section\", { id: \"main\" })"));
}

#[test]
fn type_eraser_strips_abstract_class_modifier() {
    let js = emit_to_string("abstract class Foo { abstract bar(): void; }");
    assert!(!js.contains("abstract"));
    assert!(js.contains("class Foo"));
    assert!(js.contains("bar()"));
}

#[test]
fn type_eraser_strips_readonly_modifier() {
    let js = emit_to_string("class Foo { readonly x: number = 1; }");
    assert!(!js.contains("readonly"));
    assert!(js.contains("x = 1;"));
}

#[test]
fn type_eraser_strips_override_modifier() {
    let js = emit_to_string("class Foo { override m(): void {} }");
    assert!(!js.contains("override"));
    assert!(js.contains("m()"));
}

#[test]
fn type_eraser_strips_implements_clause() {
    let js = emit_to_string("interface I { x: number; }\nclass Foo implements I { x = 1; }");
    assert!(!js.contains("implements"));
    assert!(!js.contains("interface"));
    assert!(js.contains("class Foo"));
    assert!(js.contains("x = 1;"));
}

#[test]
fn type_eraser_keeps_extends_strips_implements() {
    let js =
        emit_to_string("class Base {}\ninterface I {}\nclass Foo extends Base implements I {}");
    assert!(js.contains("extends Base"));
    assert!(!js.contains("implements"));
}

#[test]
fn type_eraser_strips_declare_keyword() {
    let js = emit_to_string("declare const x: number;\nlet y = x;");
    assert!(!js.contains("declare"));
}

#[test]
fn type_eraser_strips_type_assertion() {
    let js = emit_to_string("let x = <number>5;");
    assert!(
        !js.contains("<number>"),
        "type assertion <number> should be erased, got {js:?}"
    );
    assert!(js.contains("5;"));
}

#[test]
fn import_elision_import_type_named() {
    let js = emit_to_string("import type { Foo } from \"./bar\";\nlet x = 1;");
    assert!(!js.contains("import"));
    assert!(!js.contains("Foo"));
    assert!(js.contains("let x = 1;"));
}

#[test]
fn import_elision_import_type_default() {
    let js = emit_to_string("import type Foo from \"./bar\";\nlet x = 1;");
    assert!(!js.contains("import"));
    assert!(!js.contains("Foo"));
    assert!(js.contains("let x = 1;"));
}

#[test]
fn import_elision_import_type_namespace() {
    let js = emit_to_string("import type * as ns from \"./bar\";\nlet x = 1;");
    assert!(!js.contains("import"));
    assert!(!js.contains("require"));
    assert!(js.contains("let x = 1;"));
}

#[test]
fn import_elision_mixed_named_bindings() {
    let js = emit_to_string("import { type Foo, Bar } from \"./bar\";\nlet x = Bar;");
    assert!(
        !js.contains("Foo"),
        "type-only binding Foo should be elided, got {js:?}"
    );
    assert!(
        js.contains("Bar"),
        "value binding Bar should be retained, got {js:?}"
    );
    assert!(js.contains("from \"./bar\";"));
}

#[test]
fn import_elision_mixed_named_bindings_trailing() {
    let js = emit_to_string("import { Bar, type Foo } from \"./bar\";\nlet x = Bar;");
    assert!(!js.contains("Foo"));
    assert!(js.contains("Bar"));
    assert!(js.contains("from \"./bar\";"));
}

#[test]
fn import_elision_mixed_default_and_type_named() {
    let js = emit_to_string("import Foo, { type Bar } from \"./bar\";\nlet x = Foo;");
    assert!(js.contains("Foo"));
    assert!(!js.contains("Bar"));
    assert!(js.contains("from \"./bar\";"));
}

#[test]
fn import_elision_preserves_value_import() {
    let js = emit_to_string("import { foo } from \"./bar\";\nlet x = foo;");
    assert!(js.contains("import { foo }"));
    assert!(js.contains("from \"./bar\";"));
}

#[test]
fn import_elision_all_inline_type_only() {
    let js = emit_to_string("import { type Foo, type Bar } from \"./bar\";\nlet x = 1;");
    assert!(!js.contains("import"));
    assert!(!js.contains("Foo"));
    assert!(!js.contains("Bar"));
    assert!(js.contains("let x = 1;"));
}
