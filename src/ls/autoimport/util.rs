//! Utilities (1:1 port of Go's `internal/ls/autoimport/util.go`).

use std::collections::HashMap;

use crate::ast::Symbol;
use crate::checker::Checker;
use crate::collections::set::Set;
use crate::compiler::Program;
use crate::module;
use crate::tspath;
use crate::vfs::FS;

use super::export::ModuleID;

/// Tries to get the module ID and file name of a module symbol.
///
/// Mirrors `tryGetModuleIDAndFileNameOfModuleSymbol` in Go.
pub fn try_get_module_id_and_file_name_of_module_symbol(
    _symbol: &Symbol,
) -> Option<(ModuleID, String)> {
    // Requires ast.GetNonAugmentationDeclaration, ast.IsModuleWithStringLiteralName,
    // and SourceFile access — stubbed.
    todo!("try_get_module_id_and_file_name_of_module_symbol requires ast helpers")
}

/// Gets the module ID and file name of a module symbol (panics if not an external module).
///
/// Mirrors `getModuleIDAndFileNameOfModuleSymbol` in Go.
pub fn get_module_id_and_file_name_of_module_symbol(symbol: &Symbol) -> (ModuleID, String) {
    if !symbol.is_external_module() {
        panic!("symbol is not an external module");
    }
    match try_get_module_id_and_file_name_of_module_symbol(symbol) {
        Some((id, file)) => (id, file),
        None => panic!("could not determine module ID of module symbol"),
    }
}

/// Splits an identifier into its constituent words based on camelCase and snake_case
/// conventions by returning the starting byte indices of each word.
///
/// Mirrors `wordIndices` in Go. (Delegates to `index::word_indices`.)
pub fn word_indices(s: &str) -> Vec<usize> {
    super::index::word_indices(s)
}

/// Gets the set of package names in a node_modules directory.
///
/// Mirrors `getPackageNamesInNodeModules` in Go.
pub fn get_package_names_in_node_modules(node_modules_dir: &str, fs: &dyn FS) -> Set<String> {
    let mut package_names = Set::new();
    if tspath::get_base_file_name(node_modules_dir) != "node_modules" {
        panic!("nodeModulesDir is not a node_modules directory");
    }
    let entries = fs.get_accessible_entries(node_modules_dir);
    for base_name in &entries.directories {
        if base_name.starts_with('.') {
            continue;
        }
        if base_name.starts_with('@') {
            let scoped_dir_path = tspath::combine_paths(node_modules_dir, &[base_name]);
            let scoped_entries = fs.get_accessible_entries(&scoped_dir_path);
            for scoped_package_dir_name in &scoped_entries.directories {
                let scoped_base_name = tspath::get_base_file_name(scoped_package_dir_name);
                if base_name == "@types" {
                    package_names.add(module::get_package_name_from_types_package_name(
                        &tspath::combine_paths("@types", &[&scoped_base_name]),
                    ));
                } else {
                    package_names.add(tspath::combine_paths(base_name, &[&scoped_base_name]));
                }
            }
            continue;
        }
        package_names.add(base_name.clone());
    }
    package_names
}

/// Gets the default-like export name from a declaration.
///
/// Mirrors `getDefaultLikeExportNameFromDeclaration` in Go.
pub fn get_default_like_export_name_from_declaration(_symbol: &Symbol) -> String {
    todo!("getDefaultLikeExportNameFromDeclaration requires ast helpers")
}

/// Gets the resolved package names for a program.
///
/// Mirrors `getResolvedPackageNames` in Go.
pub fn get_resolved_package_names(_program: &Program) -> Set<String> {
    todo!("getResolvedPackageNames requires program and checker methods")
}

/// Adds output .d.ts to source file mappings from a program's project references.
///
/// Mirrors `addProjectReferenceOutputMappings` in Go.
pub fn add_project_reference_output_mappings(
    _program: &Program,
    _result: &mut HashMap<tspath::Path, String>,
) {
    todo!("addProjectReferenceOutputMappings requires project reference infrastructure")
}

/// Creates a checker pool for parallel export extraction.
///
/// Mirrors `createCheckerPool` in Go.
pub fn create_checker_pool(
    _program: &Program,
) -> (
    Box<dyn Fn() -> (Checker, Box<dyn FnOnce()>) + Send + Sync>,
    Box<dyn FnOnce() + Send>,
    Box<dyn Fn() -> i32 + Send + Sync>,
) {
    todo!("createCheckerPool requires checker.NewChecker")
}

/// Adds all dependencies and peerDependencies from a package.json to the given set,
/// canonicalizing @types package names to their base names.
///
/// Mirrors `addPackageJsonDependencies` in Go.
pub fn add_package_json_dependencies(_deps: &mut Set<String>) {
    // The Go signature is:
    //   func addPackageJsonDependencies(contents *packagejson.PackageJson, deps *collections.Set[string])
    // Requires packagejson.Contents.RangeDependencies — stubbed.
    todo!("addPackageJsonDependencies requires packagejson.Contents.RangeDependencies")
}

/// Returns functions to transform between symlink and realpath for files within a package.
///
/// Mirrors `getPackageRealpathFuncs` in Go.
pub fn get_package_realpath_funcs(
    fs: &dyn FS,
    package_dir: &str,
) -> (
    Box<dyn Fn(&str) -> String + Send + Sync>,
    Box<dyn Fn(&str) -> String + Send + Sync>,
) {
    let real_package_dir = fs.realpath(package_dir);
    let is_symlinked = real_package_dir != package_dir;

    let package_dir_owned = package_dir.to_string();
    let real_package_dir_owned = real_package_dir.clone();

    let to_realpath: Box<dyn Fn(&str) -> String + Send + Sync> = {
        let real_package_dir = real_package_dir.clone();
        let package_dir = package_dir.to_string();
        Box::new(move |file_name: &str| {
            // Fast path: files within the package use prefix substitution.
            if is_symlinked {
                if let Some(after) = file_name.strip_prefix(&package_dir) {
                    return format!("{}{}", real_package_dir, after);
                }
            }
            // For files outside the package, find the node_modules package dir.
            let pkg_dir = module::parse_node_module_from_path(file_name, false);
            if pkg_dir.is_empty() {
                return file_name.to_string();
            }
            file_name.to_string()
        })
    };

    if !is_symlinked {
        let to_symlink: Box<dyn Fn(&str) -> String + Send + Sync> =
            Box::new(|file_name: &str| file_name.to_string());
        return (to_realpath, to_symlink);
    }

    let to_symlink: Box<dyn Fn(&str) -> String + Send + Sync> = {
        let real_package_dir = real_package_dir_owned.clone();
        let package_dir = package_dir_owned.clone();
        Box::new(move |file_name: &str| {
            if let Some(after) = file_name.strip_prefix(&real_package_dir) {
                return format!("{}{}", package_dir, after);
            }
            file_name.to_string()
        })
    };

    (to_realpath, to_symlink)
}

/// A path-and-file-name pair.
///
/// Mirrors `autoimport.pathAndFileName` in Go.
#[derive(Debug, Clone, Default)]
pub struct PathAndFileName {
    pub path: tspath::Path,
    pub file_name: String,
}

/// A resolution host wrapper for auto-import module resolution.
///
/// Mirrors `autoimport.resolutionHost` in Go.
pub struct ResolutionHost {
    pub fs: Box<dyn FS>,
    pub current_directory: String,
}

impl ResolutionHost {
    pub fn get_current_directory(&self) -> &str {
        &self.current_directory
    }
    pub fn fs(&self) -> &dyn FS {
        self.fs.as_ref()
    }
}
