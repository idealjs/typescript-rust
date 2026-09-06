use crate::ast::*;
use crate::core::tristate::Tristate;
use crate::tspath::is_external_module_name_relative;
use std::sync::Arc;

const EXCLUSIVELY_PREFIXED_NODE_CORE_MODULES: &[&str] = &[
    "node:sea",
    "node:sqlite",
    "node:test",
    "node:diagnostics_channel",
];

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

pub fn collect_external_module_references(file: &mut SourceFile) {
    let statements: Vec<Arc<Node>> = if let NodeData::SourceFile(d) = &file.node.data {
        d.statements.nodes.clone()
    } else {
        return;
    };

    for stmt in &statements {
        collect_module_references(file, stmt, false);
    }

}

fn collect_module_references(file: &mut SourceFile, node: &Arc<Node>, in_ambient_module: bool) {

    if let Some(module_name_expr) = get_external_module_name(node) {
        if is_string_literal(&module_name_expr) {
            let module_name = module_name_expr.text();
            if !module_name.is_empty()
                && (!in_ambient_module || !is_external_module_name_relative(module_name))
            {
                file.imports.push(module_name_expr.clone());

                if file.uses_uri_style_node_core_modules != Tristate::True
                    && !file.is_declaration_file
                {
                    if module_name.starts_with("node:")
                        && !EXCLUSIVELY_PREFIXED_NODE_CORE_MODULES.contains(&module_name)
                    {

                        file.uses_uri_style_node_core_modules = Tristate::True;
                    } else if file.uses_uri_style_node_core_modules == Tristate::Unknown
                        && UNPREFIXED_NODE_CORE_MODULES.contains(&module_name)
                    {

                        file.uses_uri_style_node_core_modules = Tristate::False;
                    }
                }
            }
        }
        return;
    }

    if is_module_declaration(node) && is_ambient_module(node) {
        let is_ambient = in_ambient_module
            || node.has_syntactic_modifier(ModifierFlags::Ambient)
            || file.is_declaration_file;

        if is_ambient {
            if let NodeData::ModuleDeclaration(d) = &node.data {

                let raw_name = d.name.text();
                let name_text = if raw_name.len() >= 2
                    && ((raw_name.starts_with('"') && raw_name.ends_with('"'))
                        || (raw_name.starts_with('\'') && raw_name.ends_with('\'')))
                {
                    &raw_name[1..raw_name.len() - 1]
                } else {
                    raw_name
                };

                if is_external_module(file)
                    || (in_ambient_module && !is_external_module_name_relative(name_text))
                {
                    file.module_augmentations.push(d.name.clone());
                } else if !in_ambient_module {
                    file.ambient_module_names.push(name_text.to_string());

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

fn get_external_module_name(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::ImportDeclaration(d) => Some(d.module_specifier.clone()),
        NodeData::ExportDeclaration(d) => d.module_specifier.clone(),
        NodeData::ImportEqualsDeclaration(d) => {

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

fn is_ambient_module(node: &Arc<Node>) -> bool {
    if node.kind != SyntaxKind::ModuleDeclaration {
        return false;
    }
    if let NodeData::ModuleDeclaration(d) = &node.data {

        d.name.kind == SyntaxKind::StringLiteral
            || (d.name.kind == SyntaxKind::Identifier && d.name.text() == "global")
    } else {
        false
    }
}

fn is_external_module(file: &SourceFile) -> bool {
    file.external_module_indicator.is_some()
}

pub fn set_external_module_indicator(file: &mut SourceFile) {
    if file.script_kind == ScriptKind::Json {
        return;
    }

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

    if file.is_declaration_file {
        return;
    }

}

fn is_external_module_indicator_node(node: &Arc<Node>) -> bool {
    if node.has_syntactic_modifier(ModifierFlags::Export) {
        return true;
    }
    match &node.data {
        NodeData::ImportDeclaration(_)
        | NodeData::ExportAssignment(_)
        | NodeData::ExportDeclaration(_) => true,
        NodeData::ImportEqualsDeclaration(d) => {

            d.module_reference.kind == SyntaxKind::ExternalModuleReference
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
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
}
