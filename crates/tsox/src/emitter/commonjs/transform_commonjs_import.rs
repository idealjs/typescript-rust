#![allow(unused_imports)]

use super::*;

pub(crate) fn transform_commonjs_import(stmt: &Node, source: &str) -> Option<String> {
    let import_data = match &stmt.data {
        NodeData::ImportDeclaration(d) => d,
        _ => return None,
    };

    let specifier = &import_data.module_specifier;
    let specifier_text = &source[specifier.pos()..specifier.end()];

    let clause = match &import_data.import_clause {
        None => return Some(format!("require({specifier_text});")),
        Some(c) => c,
    };
    let clause_data = match &clause.data {
        NodeData::ImportClause(d) => d,
        _ => return Some(format!("require({specifier_text});")),
    };

    if clause_data.phase_modifier == Some(SyntaxKind::TypeKeyword) {
        return Some(String::new());
    }

    if let Some(bindings) = &clause_data.named_bindings {
        if let NodeData::NamespaceImport(ns_data) = &bindings.data {
            if let NodeData::Identifier(ident) = &ns_data.name.data {
                return Some(format!(
                    "const {} = require({});",
                    ident.text, specifier_text
                ));
            }
        }
    }

    let mut parts: Vec<String> = Vec::new();

    if let Some(name) = &clause_data.name {
        if let NodeData::Identifier(ident) = &name.data {
            parts.push(format!("default: {}", ident.text));
        }
    }

    if let Some(bindings) = &clause_data.named_bindings {
        if let NodeData::NamedImports(named) = &bindings.data {
            for spec in named.elements.iter() {
                if let NodeData::ImportSpecifier(spec_data) = &spec.data {
                    if spec_data.is_type_only {
                        continue;
                    }
                    if let Some(prop_name) = &spec_data.property_name {
                        if let (
                            NodeData::Identifier(prop_ident),
                            NodeData::Identifier(name_ident),
                        ) = (&prop_name.data, &spec_data.name.data)
                        {
                            parts.push(format!("{}: {}", prop_ident.text, name_ident.text));
                        }
                    } else if let NodeData::Identifier(name_ident) = &spec_data.name.data {
                        parts.push(name_ident.text.clone());
                    }
                }
            }
        }
    }

    if parts.is_empty() {
        return Some(format!("require({specifier_text});"));
    }

    Some(format!(
        "const {{ {} }} = require({});",
        parts.join(", "),
        specifier_text
    ))
}

pub(crate) fn transform_commonjs_export(stmt: &Node, source: &str) -> Option<String> {
    match &stmt.data {
        NodeData::ExportDeclaration(d) => {
            if d.is_type_only {
                return Some(String::new());
            }

            let specifier_text = d
                .module_specifier
                .as_ref()
                .map(|spec| source[spec.pos()..spec.end()].to_string());

            match d.export_clause.as_ref().map(|c| (&c.kind, c)) {
                Some((SyntaxKind::NamedExports, clause_node)) => {
                    if let NodeData::NamedExports(named) = &clause_node.data {
                        let mut lines: Vec<String> = Vec::new();

                        if let Some(spec) = &specifier_text {
                            let mut import_parts: Vec<String> = Vec::new();
                            for spec_node in named.elements.iter() {
                                if let NodeData::ExportSpecifier(spec_data) = &spec_node.data {
                                    if let NodeData::Identifier(name_ident) = &spec_data.name.data {
                                        import_parts.push(name_ident.text.clone());
                                    }
                                }
                            }
                            if !import_parts.is_empty() {
                                lines.push(format!(
                                    "const {{ {} }} = require({});",
                                    import_parts.join(", "),
                                    spec
                                ));
                            }
                        }

                        for spec_node in named.elements.iter() {
                            if let NodeData::ExportSpecifier(spec_data) = &spec_node.data {
                                let (local_name, export_name) =
                                    if let Some(prop_name) = &spec_data.property_name {
                                        match (&prop_name.data, &spec_data.name.data) {
                                            (NodeData::Identifier(p), NodeData::Identifier(n)) => {
                                                (p.text.clone(), n.text.clone())
                                            }
                                            _ => continue,
                                        }
                                    } else if let NodeData::Identifier(name_ident) =
                                        &spec_data.name.data
                                    {
                                        (name_ident.text.clone(), name_ident.text.clone())
                                    } else {
                                        continue;
                                    };
                                lines.push(format!("exports.{export_name} = {local_name};"));
                            }
                        }
                        return Some(lines.join("\n"));
                    }
                    Some(String::new())
                }
                Some((SyntaxKind::NamespaceExport, clause_node)) => {
                    if let NodeData::NamespaceExport(ns_data) = &clause_node.data {
                        if let NodeData::Identifier(ident) = &ns_data.name.data {
                            if let Some(spec) = &specifier_text {
                                return Some(format!(
                                    "const {n} = require({s});\nexports.{n} = {n};",
                                    n = ident.text,
                                    s = spec
                                ));
                            }
                        }
                    }
                    Some(String::new())
                }
                None => {
                    if let Some(spec) = &specifier_text {
                        return Some(format!(
                            "Object.keys(require({s})).forEach(function(k) {{ if (k !== \"default\") exports[k] = require({s})[k]; }});",
                            s = spec
                        ));
                    }
                    Some(String::new())
                }
                _ => Some(String::new()),
            }
        }
        NodeData::ExportAssignment(d) => {
            let expr_source = source[d.expression.pos()..d.expression.end()].to_string();
            if d.is_export_equals {
                Some(format!("module.exports = {expr_source};"))
            } else {
                Some(format!("exports.default = {expr_source};"))
            }
        }
        _ => None,
    }
}

pub(crate) fn transform_commonjs_export_declaration(stmt: &Node, _source: &str) -> Option<String> {
    let modifiers = stmt.modifiers()?;
    if !modifiers.modifier_flags.contains(ModifierFlags::Export) {
        return None;
    }
    let is_default = modifiers.modifier_flags.contains(ModifierFlags::Default);

    match &stmt.data {
        NodeData::VariableStatement(d) => {
            let decl_list = &d.declaration_list;
            let list_data = match &decl_list.data {
                NodeData::VariableDeclarationList(ld) => ld,
                _ => return None,
            };
            let mut lines: Vec<String> = Vec::new();
            for decl in list_data.declarations.iter() {
                if let NodeData::VariableDeclaration(decl_data) = &decl.data {
                    if let NodeData::Identifier(ident) = &decl_data.name.data {
                        if is_default {
                            lines.push(format!("exports.default = {};", ident.text));
                        } else {
                            lines.push(format!("exports.{n} = {n};", n = ident.text));
                        }
                    }
                }
            }
            if lines.is_empty() {
                None
            } else {
                Some(lines.join("\n"))
            }
        }
        NodeData::FunctionDeclaration(d) => {
            let name = d.name.as_ref()?;
            if let NodeData::Identifier(ident) = &name.data {
                if is_default {
                    Some(format!("exports.default = {};", ident.text))
                } else {
                    Some(format!("exports.{n} = {n};", n = ident.text))
                }
            } else {
                None
            }
        }
        NodeData::ClassDeclaration(d) => {
            let name = d.name.as_ref()?;
            if let NodeData::Identifier(ident) = &name.data {
                if is_default {
                    Some(format!("exports.default = {};", ident.text))
                } else {
                    Some(format!("exports.{n} = {n};", n = ident.text))
                }
            } else {
                None
            }
        }
        NodeData::EnumDeclaration(d) => {
            if let NodeData::Identifier(ident) = &d.name.data {
                Some(format!("exports.{n} = {n};", n = ident.text))
            } else {
                None
            }
        }
        _ => None,
    }
}

pub(crate) fn is_type_only_statement(node: &Node) -> bool {
    match &node.data {
        NodeData::InterfaceDeclaration(_) => true,
        NodeData::TypeAliasDeclaration(_) => true,

        NodeData::ImportDeclaration(d) => is_type_only_import(d),

        NodeData::NamespaceExportDeclaration(_) => true,
        _ => false,
    }
}

pub(crate) fn is_type_only_import(d: &ImportDeclarationData) -> bool {
    let clause = match &d.import_clause {
        Some(c) => c,
        None => return false,
    };
    let cd = match &clause.data {
        NodeData::ImportClause(cd) => cd,
        _ => return false,
    };

    if cd.phase_modifier == Some(SyntaxKind::TypeKeyword) {
        return true;
    }

    if cd.name.is_none() {
        if let Some(bindings) = &cd.named_bindings {
            if let NodeData::NamedImports(named) = &bindings.data {
                return !named.elements.is_empty()
                    && named
                        .elements
                        .iter()
                        .all(|spec| is_type_only_import_specifier(spec));
            }
        }
    }
    false
}

pub(crate) fn is_type_only_import_specifier(spec: &Node) -> bool {
    matches!(&spec.data, NodeData::ImportSpecifier(sd) if sd.is_type_only)
}
