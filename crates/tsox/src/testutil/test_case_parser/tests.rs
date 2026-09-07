use super::*;

#[test]
fn parse_single_file() {
    let content = "const x: number = 42;\n";
    let result = parse_test_files(content, "test.ts");
    assert_eq!(result.units.len(), 1);
    assert_eq!(result.units[0].name, "test.ts");
    assert!(result.units[0].content.contains("const x"));
}

#[test]
fn parse_multi_file() {
    let content = "\
// @filename: a.ts
export const x = 1;

// @filename: b.ts
import { x } from './a';
";
    let result = parse_test_files(content, "test.ts");
    assert_eq!(result.units.len(), 2);
    assert_eq!(result.units[0].name, "a.ts");
    assert_eq!(result.units[1].name, "b.ts");
}

#[test]
fn parse_compiler_settings() {
    let content = "\
// @strict: true
// @target: esnext
const x = 1;
";
    let settings = extract_compiler_settings(content);
    assert_eq!(settings.get("strict"), Some(&"true".to_string()));
    assert_eq!(settings.get("target"), Some(&"esnext".to_string()));
}

#[test]
fn parse_tsconfig_file() {
    let content = "\
// @filename: tsconfig.json
{ \"compilerOptions\": { \"strict\": true } }

// @filename: main.ts
const x = 1;
";
    let result = parse_test_files(content, "test.ts");
    assert!(result.tsconfig_content.is_some());
    assert_eq!(result.units.len(), 1);
    assert_eq!(result.units[0].name, "main.ts");
}

#[test]
fn parse_current_directory() {
    let content = "\
// @currentDirectory: /project/src
const x = 1;
";
    let result = parse_test_files(content, "test.ts");
    assert_eq!(result.current_directory, "/project/src");
}
