use tsox_harness::case_parser::{extract_settings, split_units};

#[test]
fn extracts_module_and_strict_directives() {
    let src = "// @module: commonjs\n// @strict: true\n// @noImplicitAny: false\nlet x = 1;\n";
    let s = extract_settings(src);
    assert_eq!(s.get("module"), Some(&"commonjs".to_string()));
    assert_eq!(s.get("strict"), Some(&"true".to_string()));
    assert_eq!(s.get("noimplicitany"), Some(&"false".to_string()));
}

#[test]
fn split_single_file_no_directive() {
    let src = "// @module: commonjs\nlet x = 1;\n";
    let parsed = split_units(src, "a.ts");
    assert_eq!(parsed.units.len(), 1);
    assert_eq!(parsed.units[0].name, "a.ts");
    assert_eq!(parsed.units[0].content, "let x = 1;\n");
    assert_eq!(parsed.settings.get("module"), Some(&"commonjs".to_string()));
}

#[test]
fn split_multi_file() {
    let src = "\
// @filename: a.ts
export const a = 1;
// @filename: b.ts
import { a } from './a';
console.log(a);
";
    let parsed = split_units(src, "main.ts");
    assert_eq!(parsed.units.len(), 2);
    assert_eq!(parsed.units[0].name, "a.ts");
    assert_eq!(parsed.units[1].name, "b.ts");
    assert!(parsed.units[0].content.contains("export const a"));
    assert!(parsed.units[1].content.contains("import"));
}

#[test]
fn current_directory_directive_captured() {
    let src = "// @currentdirectory: /proj/sub\n// @filename: a.ts\nlet x = 1;\n";
    let parsed = split_units(src, "a.ts");
    assert_eq!(parsed.current_directory.as_deref(), Some("/proj/sub"));
}
