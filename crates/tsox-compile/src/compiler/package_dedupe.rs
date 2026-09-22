#![allow(unused_imports)]

use super::*;

pub(crate) fn prewalk_package_first_paths(
    resolver: &tsox_tsoptions::module::Resolver,
    host: &dyn CompilerHost,
    files: &[Arc<SourceFile>],
    first_paths: &mut HashMap<tsox_tsoptions::module::PackageId, String>,
) {
    let mut visited: std::collections::HashSet<String> =
        files.iter().map(|f| f.file_name.clone()).collect();
    for file in files {
        walk_imports(resolver, host, file, &mut visited, first_paths);
    }
}

fn walk_imports(
    resolver: &tsox_tsoptions::module::Resolver,
    host: &dyn CompilerHost,
    file: &Arc<SourceFile>,
    visited: &mut std::collections::HashSet<String>,
    first_paths: &mut HashMap<tsox_tsoptions::module::PackageId, String>,
) {
    for import_node in &file.imports {
        let module_spec = import_node.text();
        if module_spec.is_empty() {
            continue;
        }
        let override_mode = import_resolution_mode_override(import_node);
        let resolution_mode = if matches!(
            override_mode,
            tsox_core::core::compiler_options::ModuleKind::None
        ) {
            tsox_tsoptions::tsoptions::implied_node_format_of_file(&file.file_name, &|p| {
                host.fs().read_file(p)
            })
        } else {
            override_mode
        };
        let (resolved, _traces) =
            resolver.resolve_module_name(module_spec, &file.file_name, resolution_mode, None);
        let Some(resolved_module) = resolved.filter(|m| m.is_resolved()) else {
            continue;
        };
        let resolved_path = host.fs().realpath(resolved_module.resolved_file_name.as_str());
        if !visited.insert(resolved_path.clone()) {
            continue;
        }
        if let Some(pid) = resolved_module.package_id {
            first_paths.entry(pid).or_insert_with(|| resolved_path.clone());
        }
        let parsed = read_and_parse(&resolved_path, host).ok().map(|(f, _)| f);
        if let Some(sub) = parsed {
            walk_imports(resolver, host, &sub, visited, first_paths);
        }
    }
}
