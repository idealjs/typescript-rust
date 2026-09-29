pub(crate) use crate::ast::*;
pub(crate) use std::sync::Arc;
pub(crate) use tsox_core::core::tristate::Tristate;
pub(crate) use tsox_core::tspath::is_external_module_name_relative;

pub const EXCLUSIVELY_PREFIXED_NODE_CORE_MODULES: &[&str] = &[
    "node:sea",
    "node:sqlite",
    "node:test",
    "node:diagnostics_channel",
];

pub const UNPREFIXED_NODE_CORE_MODULES: &[&str] = &[
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

    // Go 以 NodeFlagsPossiblyContainsDynamicImport 短路此扫描，标志在解析
    // import 调用/ImportType 时置位，扫描结果与之等价，直接执行
    let mut dynamic_specs: Vec<Arc<Node>> = Vec::new();
    for_each_dynamic_import_or_require_call(file, true, true, |_node, module_specifier| {
        dynamic_specs.push(Arc::clone(module_specifier));
        false
    });
    file.imports.append(&mut dynamic_specs);
}

pub(crate) fn collect_module_references(
    file: &mut SourceFile,
    node: &Arc<Node>,
    in_ambient_module: bool,
) {
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

pub(crate) fn get_external_module_name(node: &Arc<Node>) -> Option<Arc<Node>> {
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

pub(crate) fn is_ambient_module(node: &Arc<Node>) -> bool {
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

pub(crate) fn is_external_module(file: &SourceFile) -> bool {
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

pub(crate) fn is_external_module_indicator_node(node: &Arc<Node>) -> bool {
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

/// Go getCannotResolveModuleNameErrorForSpecificModule：node 核心模块缺失时
/// 给安装 @types/node 的专用提示（types 含 * 时走 2580 变体）
pub fn cannot_resolve_module_error(
    options: &tsox_core::core::compiler_options::CompilerOptions,
    module_spec: &str,
) -> (&'static tsox_core::diagnostics::Message, Vec<String>) {
    use tsox_core::diagnostics::messages_generated as msg;
    let is_node_core = UNPREFIXED_NODE_CORE_MODULES.contains(&module_spec)
        || module_spec
            .strip_prefix("node:")
            .is_some_and(|r| UNPREFIXED_NODE_CORE_MODULES.contains(&r))
        || EXCLUSIVELY_PREFIXED_NODE_CORE_MODULES.contains(&module_spec);
    if is_node_core {
        if options.types.iter().any(|t| t == "*") {
            return (
                &msg::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_NODE_TRY_NPM_I_SAVE_DEV_TYPES_SLASHNODE,
                vec![module_spec.to_string()],
            );
        }
        return (
            &msg::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_NODE_TRY_NPM_I_SAVE_DEV_TYPES_SLASHNODE_AND_THEN_ADD_NODE_TO_THE_TYPES_FIELD_IN_YOUR_TSCONFIG,
            vec![module_spec.to_string()],
        );
    }
    (
        &tsox_core::diagnostics::CANNOT_FIND_MODULE_0_OR_ITS_CORRESPONDING_TYPE_DECLARATIONS,
        vec![module_spec.to_string()],
    )
}
