use super::*;
use crate::parser::Parser;

pub(crate) fn parse_and_collect(source: &str) -> SourceFile {
    let (file, _diags) =
        Parser::parse_source_file_text_with_diagnostics("test.ts", source.to_string());
    file
}

#[test]
pub(crate) fn test_import_declaration_collected() {
    let file = parse_and_collect(r#"import { foo } from "bar";"#);
    assert!(file.external_module_indicator.is_some());
    assert_eq!(file.imports.len(), 1);
    assert_eq!(file.imports[0].text(), "bar");
}

#[test]
pub(crate) fn test_export_declaration_collected() {
    let file = parse_and_collect(r#"export { foo } from "bar";"#);
    assert!(file.external_module_indicator.is_some());
    assert_eq!(file.imports.len(), 1);
    assert_eq!(file.imports[0].text(), "bar");
}

#[test]
pub(crate) fn test_export_statement_makes_module() {
    let file = parse_and_collect("export const x = 42;");
    assert!(file.external_module_indicator.is_some());
    assert_eq!(file.imports.len(), 0);
}

#[test]
pub(crate) fn test_plain_script_not_module() {
    let file = parse_and_collect("const x = 42;");
    assert!(file.external_module_indicator.is_none());
    assert_eq!(file.imports.len(), 0);
}

#[test]
pub(crate) fn test_relative_import_in_ambient_not_collected() {
    let source = r#"declare module "foo" { import x from "./relative"; }"#;
    let file = parse_and_collect(source);

    assert_eq!(file.ambient_module_names.len(), 1);
    assert_eq!(file.ambient_module_names[0], "foo");
    assert_eq!(file.imports.len(), 0);
}

#[test]
pub(crate) fn test_non_relative_import_in_ambient_collected() {
    let source = r#"declare module "foo" { import x from "pkg"; }"#;
    let file = parse_and_collect(source);
    assert_eq!(file.ambient_module_names.len(), 1);
    assert_eq!(file.imports.len(), 1);
    assert_eq!(file.imports[0].text(), "pkg");
}

#[test]
pub(crate) fn test_node_core_module_tracking() {
    let file = parse_and_collect(r#"import { readFile } from "fs";"#);
    assert_eq!(file.uses_uri_style_node_core_modules, Tristate::False);
}

#[test]
pub(crate) fn test_uri_style_node_module() {
    let file = parse_and_collect(r#"import { readFile } from "node:fs";"#);
    assert_eq!(file.uses_uri_style_node_core_modules, Tristate::True);
}

#[test]
pub(crate) fn test_non_node_module_unknown() {
    let file = parse_and_collect(r#"import { foo } from "some-pkg";"#);
    assert_eq!(file.uses_uri_style_node_core_modules, Tristate::Unknown);
}

#[test]
pub(crate) fn test_module_augmentation_in_external_module() {
    let source = r#"import { x } from "a";
declare module "foo" { const y: number; }
"#;
    let file = parse_and_collect(source);
    assert!(file.external_module_indicator.is_some());
    assert_eq!(file.module_augmentations.len(), 1);
    assert_eq!(file.ambient_module_names.len(), 0);
}

#[test]
pub(crate) fn test_ambient_module_in_script() {
    let source = r#"declare module "foo" { const y: number; }"#;
    let file = parse_and_collect(source);
    assert!(file.external_module_indicator.is_none());
    assert_eq!(file.ambient_module_names.len(), 1);
    assert_eq!(file.module_augmentations.len(), 0);
}
