//! External module reference collection, ported from
//! `internal/parser/references.go`.
//!
//! After parsing, walks the source file's top-level statements to collect:
//! - `imports`: module specifier expressions from import/export statements
//! - `module_augmentations`: `declare module "name"` in external module files
//! - `ambient_module_names`: `declare module "name"` in non-module files
//!
//! Also sets `uses_uri_style_node_core_modules` based on `node:` prefixes.

use crate::ast::*;
use crate::core::tristate::Tristate;
use crate::tspath::is_external_module_name_relative;
use std::sync::Arc;

/// Node core modules that are commonly used with a `node:` prefix.
/// Mirrors Go's `core.ExclusivelyPrefixedNodeCoreModules`.
const EXCLUSIVELY_PREFIXED_NODE_CORE_MODULES: &[&str] = &[
    "node:sea",
    "node:sqlite",
    "node:test",
    "node:diagnostics_channel",
];

/// Node core modules that can be used with or without the `node:` prefix.
/// Mirrors Go's `core.UnprefixedNodeCoreModules`.
const UNPREFIXED_NODE_CORE_MODULES: &[&str] = &[
    "assert",
    "buffer",
    "child_process",
    "cluster",
    "console",
    "constants",
    "crypto",
    "dgram",
    "dns",
    "domain",
    "events",
    "fs",
    "http",
    "http2",
    "https",
    "inspector",
    "module",
    "net",
    "os",
    "path",
    "process",
    "punycode",
    "querystring",
    "readline",
    "repl",
    "stream",
    "string_decoder",
    "sys",
    "timers",
    "tls",
    "trace_events",
    "tty",
    "url",
    "util",
    "v8",
    "vm",
    "wasi",
    "worker_threads",
    "zlib",
];

/// Collect external module references from the source file's statements.
///
/// Mirrors Go's `collectExternalModuleReferences` (`references.go:11-22`).
/// Walks top-level statements for import/export declarations and ambient
/// module declarations, populating `imports`, `module_augmentations`, and
/// `ambient_module_names`.
pub fn collect_external_module_references(file: &mut SourceFile) {
    let statements: Vec<Arc<Node>> = if let NodeData::SourceFile(d) = &file.node.data {
        d.statements.nodes.clone()
    } else {
        return;
    };

    for stmt in &statements {
        collect_module_references(file, stmt, false);
    }

    // Dynamic import / require call collection is skipped for now — it's only
    // needed for JS files with `require()` calls and `PossiblyContainsDynamicImport`
    // flag tracking. Mirrors Go's `ForEachDynamicImportOrRequireCall` which is
    // gated on `NodeFlagsPossiblyContainsDynamicImport || IsInJSFile`.
}

/// Collect module references from a single statement.
///
/// Mirrors Go's `collectModuleReferences` (`references.go:24-70`).
fn collect_module_references(
    file: &mut SourceFile,
    node: &Arc<Node>,
    in_ambient_module: bool,
) {
    // Check if this is an import or re-export statement
    if let Some(module_name_expr) = get_external_module_name(node) {
        if is_string_literal(&module_name_expr) {
            let module_name = module_name_expr.text();
            if !module_name.is_empty()
                && (!in_ambient_module || !is_external_module_name_relative(module_name))
            {
                file.imports.push(module_name_expr.clone());

                // Track URI-style node core module usage
                if file.uses_uri_style_node_core_modules != Tristate::True
                    && !file.is_declaration_file
                {
                    if module_name.starts_with("node:")
                        && !EXCLUSIVELY_PREFIXED_NODE_CORE_MODULES.contains(&module_name)
                    {
                        // `node:` prefix takes precedence
                        file.uses_uri_style_node_core_modules = Tristate::True;
                    } else if file.uses_uri_style_node_core_modules == Tristate::Unknown
                        && UNPREFIXED_NODE_CORE_MODULES.contains(&module_name)
                    {
                        // Found an unprefixed node core module import
                        file.uses_uri_style_node_core_modules = Tristate::False;
                    }
                }
            }
        }
        return;
    }

    // Check for ambient module declarations (`declare module "name"`)
    if is_module_declaration(node) && is_ambient_module(node) {
        let is_ambient = in_ambient_module
            || node.has_syntactic_modifier(ModifierFlags::Ambient)
            || file.is_declaration_file;

        if is_ambient {
            if let NodeData::ModuleDeclaration(d) = &node.data {
                // `parse_string_literal_name` stores raw text (with quotes);
                // strip them to match Go's `scanner.TokenValue()` behavior.
                let raw_name = d.name.text();
                let name_text = if raw_name.len() >= 2
                    && ((raw_name.starts_with('"') && raw_name.ends_with('"'))
                        || (raw_name.starts_with('\'') && raw_name.ends_with('\'')))
                {
                    &raw_name[1..raw_name.len() - 1]
                } else {
                    raw_name
                };

                // Module augmentation vs ambient module name
                if is_external_module(file) || (in_ambient_module && !is_external_module_name_relative(name_text)) {
                    file.module_augmentations.push(d.name.clone());
                } else if !in_ambient_module {
                    file.ambient_module_names.push(name_text.to_string());

                    // Recurse into ambient module body
                    if let Some(body) = &d.body {
                        if let NodeData::ModuleBlock(block) = &body.data {
                            for stmt in &block.statements.nodes {
                                collect_module_references(file, stmt, true);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Get the external module name (module specifier) from an import/export
/// statement. Returns `None` for non-import/export nodes.
///
/// Mirrors Go's `ast.GetExternalModuleName`.
fn get_external_module_name(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::ImportDeclaration(d) => Some(d.module_specifier.clone()),
        NodeData::ExportDeclaration(d) => d.module_specifier.clone(),
        NodeData::ImportEqualsDeclaration(d) => {
            // `import x = require("mod")` — only count external module references
            if d.module_reference.kind == SyntaxKind::ExternalModuleReference {
                if let NodeData::ExternalModuleReference(ref_data) = &d.module_reference.data {
                    return Some(ref_data.expression.clone());
                }
            }
            None
        }
        _ => None,
    }
}

/// Whether the node is an ambient module declaration (`declare module "name"`
/// or `declare global`).
///
/// Mirrors Go's `ast.IsAmbientModule` (`utilities.go:1633-1635`): checks that
/// the node is a ModuleDeclaration with a StringLiteral name (like `"foo"`)
/// or a global scope augmentation (`declare global`).
fn is_ambient_module(node: &Arc<Node>) -> bool {
    if node.kind != SyntaxKind::ModuleDeclaration {
        return false;
    }
    if let NodeData::ModuleDeclaration(d) = &node.data {
        // `declare module "foo"` — name is a StringLiteral
        // `declare global` — name is an Identifier with text "global"
        d.name.kind == SyntaxKind::StringLiteral
            || (d.name.kind == SyntaxKind::Identifier && d.name.text() == "global")
    } else {
        false
    }
}

/// Whether the source file is an external module (has import/export).
///
/// Mirrors Go's `ast.IsExternalModule(file)` which checks
/// `file.ExternalModuleIndicator != nil`.
fn is_external_module(file: &SourceFile) -> bool {
    file.external_module_indicator.is_some()
}

/// Detect and set the external module indicator on the source file.
///
/// Mirrors Go's `ast.SetExternalModuleIndicator` with `ExternalModuleIndicatorOptions{}`
/// (legacy mode: files are modules if they have imports, exports, or import.meta).
/// The `force` option (moduleDetection: force) and JSX detection are not yet
/// supported — they require compiler options plumbing.
pub fn set_external_module_indicator(file: &mut SourceFile) {
    if file.script_kind == ScriptKind::Json {
        return;
    }

    // Check for import/export statements in top-level declarations
    let statements: Vec<Arc<Node>> = if let NodeData::SourceFile(d) = &file.node.data {
        d.statements.nodes.clone()
    } else {
        return;
    };

    for stmt in &statements {
        if is_external_module_indicator_node(stmt) {
            file.external_module_indicator = Some(stmt.clone());
            return;
        }
    }

    // Declaration files without imports/exports are not modules
    if file.is_declaration_file {
        return;
    }

    // `force` and JSX detection not yet implemented — would require compiler
    // options. For now, only import/export statements trigger module-ness.
}

/// Whether a statement node marks the file as an external module.
///
/// Mirrors Go's `isAnExternalModuleIndicatorNode` (`parseoptions.go:95-99`).
fn is_external_module_indicator_node(node: &Arc<Node>) -> bool {
    if node.has_syntactic_modifier(ModifierFlags::Export) {
        return true;
    }
    match &node.data {
        NodeData::ImportDeclaration(_) | NodeData::ExportAssignment(_) | NodeData::ExportDeclaration(_) => true,
        NodeData::ImportEqualsDeclaration(d) => {
            // `import x = require("mod")` — only if it's an external module reference
            d.module_reference.kind == SyntaxKind::ExternalModuleReference
        }
        _ => false,
    }
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn parse_and_collect(source: &str) -> SourceFile {
        // `parse_source_file_text_with_diagnostics` already calls
        // `set_external_module_indicator` + `collect_external_module_references`.
        let (file, _diags) =
            Parser::parse_source_file_text_with_diagnostics("test.ts", source.to_string());
        file
    }

    #[test]
    fn test_import_declaration_collected() {
        let file = parse_and_collect(r#"import { foo } from "bar";"#);
        assert!(file.external_module_indicator.is_some());
        assert_eq!(file.imports.len(), 1);
        assert_eq!(file.imports[0].text(), "bar");
    }

    #[test]
    fn test_export_declaration_collected() {
        let file = parse_and_collect(r#"export { foo } from "bar";"#);
        assert!(file.external_module_indicator.is_some());
        assert_eq!(file.imports.len(), 1);
        assert_eq!(file.imports[0].text(), "bar");
    }

    #[test]
    fn test_export_statement_makes_module() {
        let file = parse_and_collect("export const x = 42;");
        assert!(file.external_module_indicator.is_some());
        assert_eq!(file.imports.len(), 0);
    }

    #[test]
    fn test_plain_script_not_module() {
        let file = parse_and_collect("const x = 42;");
        assert!(file.external_module_indicator.is_none());
        assert_eq!(file.imports.len(), 0);
    }

    #[test]
    fn test_relative_import_in_ambient_not_collected() {
        // `declare module "foo" { import x from "./relative"; }` — relative
        // imports inside ambient modules are skipped.
        let source = r#"declare module "foo" { import x from "./relative"; }"#;
        let file = parse_and_collect(source);
        // Ambient module name collected, but relative import skipped
        assert_eq!(file.ambient_module_names.len(), 1);
        assert_eq!(file.ambient_module_names[0], "foo");
        assert_eq!(file.imports.len(), 0);
    }

    #[test]
    fn test_non_relative_import_in_ambient_collected() {
        let source = r#"declare module "foo" { import x from "pkg"; }"#;
        let file = parse_and_collect(source);
        assert_eq!(file.ambient_module_names.len(), 1);
        assert_eq!(file.imports.len(), 1);
        assert_eq!(file.imports[0].text(), "pkg");
    }

    #[test]
    fn test_node_core_module_tracking() {
        let file = parse_and_collect(r#"import { readFile } from "fs";"#);
        assert_eq!(file.uses_uri_style_node_core_modules, Tristate::False);
    }

    #[test]
    fn test_uri_style_node_module() {
        let file = parse_and_collect(r#"import { readFile } from "node:fs";"#);
        assert_eq!(file.uses_uri_style_node_core_modules, Tristate::True);
    }

    #[test]
    fn test_non_node_module_unknown() {
        let file = parse_and_collect(r#"import { foo } from "some-pkg";"#);
        assert_eq!(file.uses_uri_style_node_core_modules, Tristate::Unknown);
    }

    #[test]
    fn test_module_augmentation_in_external_module() {
        // In an external module (has import/export), `declare module "foo"`
        // is a module augmentation
        let source = r#"import { x } from "a";
declare module "foo" { const y: number; }
"#;
        let file = parse_and_collect(source);
        assert!(file.external_module_indicator.is_some());
        assert_eq!(file.module_augmentations.len(), 1);
        assert_eq!(file.ambient_module_names.len(), 0);
    }

    #[test]
    fn test_ambient_module_in_script() {
        // In a script (no import/export), `declare module "foo"` is an
        // ambient module name
        let source = r#"declare module "foo" { const y: number; }"#;
        let file = parse_and_collect(source);
        assert!(file.external_module_indicator.is_none());
        assert_eq!(file.ambient_module_names.len(), 1);
        assert_eq!(file.module_augmentations.len(), 0);
    }
}
