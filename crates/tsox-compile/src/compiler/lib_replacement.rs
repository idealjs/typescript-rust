#![allow(unused_imports)]

use super::*;
use tsox_tsoptions::module::Resolver;

// Go fileloader getLibraryNameFromLibFileName:
// lib.dom.d.ts -> @typescript/lib-dom
// lib.dom.iterable.d.ts -> @typescript/lib-dom/iterable
// lib.es2015.symbol.wellknown.d.ts -> @typescript/lib-es2015/symbol-wellknown
pub(crate) fn library_name_from_lib_file_name(lib_file_name: &str) -> String {
    let components: Vec<&str> = lib_file_name.split('.').collect();
    let mut path = String::from("@typescript/lib-");
    if components.len() > 1 {
        path.push_str(components[1]);
    }
    let mut i = 2;
    while i < components.len() && !components[i].is_empty() && components[i] != "d" {
        if i == 2 {
            path.push('/');
        } else {
            path.push('-');
        }
        path.push_str(components[i]);
        i += 1;
    }
    path
}

// Go fileloader getInferredLibraryNameResolveFrom
fn inferred_library_name_resolve_from(options: &CompilerOptions, host: &dyn CompilerHost, lib_file_name: &str) -> String {
    let containing_directory = if !options.config_file_path.is_empty() {
        tsox_core::tspath::get_directory_path(&options.config_file_path)
    } else {
        host.current_directory().to_string()
    };
    let lookup = format!("__lib_node_modules_lookup_{lib_file_name}__.ts");
    tsox_core::tspath::combine_paths(&containing_directory, &[&lookup])
}

// Go fileloader pathForLibFile
pub(crate) fn resolve_lib_file_path(
    lib_name: &str,
    options: &CompilerOptions,
    resolver: &Resolver,
    host: &dyn CompilerHost,
) -> String {
    let mut path = tsox_core::tspath::combine_paths(host.default_library_path(), &[lib_name]);
    if options.lib_replacement.is_true() && lib_name != "lib.d.ts" {
        let library_name = library_name_from_lib_file_name(lib_name);
        let resolve_from = inferred_library_name_resolve_from(options, host, lib_name);
        let (resolved, _) = resolver.resolve_module_name(
            &library_name,
            &resolve_from,
            ModuleKind::CommonJS,
            None,
        );
        if let Some(module) = resolved {
            if module.is_resolved() {
                path = module.resolved_file_name;
            }
        }
    }
    path
}
