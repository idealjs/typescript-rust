//! Go string_completions.go 非相对模块说明符补全的移植：
//! getCompletionEntriesForNonRelativeModules 的 node_modules 祖先扫描，
//! 含 package.json exports 子路径与 typesVersions 版本重定向

use std::sync::Arc;

use tsox_compile::compiler::Program;
use tsox_frontend::ast::{ModifierFlags, NodeData, SyntaxKind};
use tsox_core::core::compiler_options::ModuleResolutionKind;
use tsox_core::tspath as tsp;
use tsox_tsoptions::packagejson;
use tsox_tsoptions::packagejson::{JsonValueType, JsonValue};

use super::completions_path_mapping::{completions_for_path_mapping, version_redirect};
use super::language_service::LanguageService;

#[derive(Default)]
pub(super) struct ModuleCompletionSet {
    names: Vec<String>,
}

impl ModuleCompletionSet {
    pub(super) fn add(&mut self, name: String) {
        if !name.is_empty() && !self.names.contains(&name) {
            self.names.push(name);
        }
    }
    pub(super) fn labels(mut self) -> Vec<String> {
        self.names.sort();
        self.names
    }
}

/// Go getStringLiteralCompletionsFromModuleNamesWorker 的非相对分支
pub(super) fn non_relative_module_labels(
    service: &LanguageService,
    program: &Arc<Program>,
    file_name: &str,
    literal_value: &str,
) -> Vec<String> {
    let fragment = literal_value.replace('\\', "/");
    let script_dir = match file_name.rfind('/') {
        Some(0) => "/".to_string(),
        Some(i) => file_name[..i].to_string(),
        None => "/".to_string(),
    };
    let mut result = ModuleCompletionSet::default();
    let fragment_directory = get_fragment_directory(&fragment);
    let project_root = service.project_path.to_string();

    // Go getAmbientModuleCompletions：环境模块声明名（fragment 前缀过滤，
    // 含 * 的模式名排除）
    for name in ambient_module_names(program) {
        if name.starts_with(&fragment) && !name.contains('*') {
            let label = if fragment_directory.is_empty() {
                name
            } else {
                let prefix = format!("{fragment_directory}/");
                name.strip_prefix(&prefix).unwrap_or(&name).to_string()
            };
            result.add(label);
        }
    }

    // Go getCompletionEntriesFromTypings：typeRoots（含默认祖先 @types），
    // `a__b` 目录名还原为 `@a/b`
    let (type_roots, _) =
        tsox_tsoptions::module::resolver::get_effective_type_roots(program.options(), &project_root);
    for root in &type_roots {
        let root = if root.starts_with('/') {
            root.clone()
        } else {
            tsp::combine_paths(&project_root, &[root])
        };
        if !service.directory_exists(&root) {
            continue;
        }
        for dir in service.get_directories(&root) {
            let base = tsp::get_base_file_name(&dir);
            if base.starts_with('.') || base == "@types" {
                continue;
            }
            let package_name = unmangle_scoped_package_name(&base);
            if !program.options().types.is_empty()
                && !program.options().types.contains(&package_name)
            {
                continue;
            }
            if fragment_directory.is_empty() {
                result.add(package_name);
            } else {
                let prefix = format!("{package_name}/");
                if let Some(remaining) = fragment_directory.strip_prefix(&prefix) {
                    directory_fragment_entries(service, program, remaining, &dir, &mut result);
                }
            }
        }
    }

    if !module_resolution_uses_node_modules(program) {
        return result.labels();
    }
    let mut ancestor = Some(script_dir);
    let mut seen_package_scope = false;
    while let Some(dir) = ancestor {
        // Go importsLookup：`#…` 形式走 package.json imports（首个包作用域一次）
        if !seen_package_scope
            && fragment.starts_with('#')
            && program.options().get_resolve_package_json_imports()
            && imports_lookup(service, program, &fragment, &dir, &mut result)
        {
            seen_package_scope = true;
        }
        let node_modules = tsp::combine_paths(&dir, &["node_modules"]);
        if fragment_directory.is_empty() {
            if service.directory_exists(&node_modules) {
                for name in service.get_directories(&node_modules) {
                    let base = tsp::get_base_file_name(&name);
                    if base != "@types" && !base.starts_with('.') && base != "bin" {
                        result.add(base);
                    }
                }
            }
        } else {
            if let Some(package_dir) = package_directory_of(&fragment, &node_modules)
                && exports_lookup(service, program, &fragment, &package_dir, &mut result)
            {
                return result.labels();
            }
            if service.directory_exists(&node_modules) {
                directory_fragment_entries(
                    service,
                    program,
                    &fragment,
                    &node_modules,
                    &mut result,
                );
            }
        }
        let parent = tsp::get_directory_path(&dir);
        ancestor = if parent == dir { None } else { Some(parent) };
    }
    result.labels()
}

/// fragment（如 `@scope/pkg/sub`）对应的 node_modules 下包目录
fn package_directory_of(fragment: &str, node_modules: &str) -> Option<String> {
    let mut components: Vec<&str> = fragment.split('/').filter(|c| !c.is_empty()).collect();
    let mut package_path = components.first().copied()?.to_string();
    components.remove(0);
    if package_path.starts_with('@') && !components.is_empty() {
        package_path = tsp::combine_paths(&package_path, &[components.remove(0)]);
    }
    Some(tsp::combine_paths(node_modules, &[&package_path]))
}

/// Go exportsOrImportsLookup：exports 存在（任意形态）即由它接管并阻断目录枚举
fn exports_lookup(
    service: &LanguageService,
    program: &Arc<Program>,
    fragment: &str,
    package_dir: &str,
    result: &mut ModuleCompletionSet,
) -> bool {
    let Some(fields) = read_package_json(service, package_dir) else {
        return false;
    };
    let exports = &fields.path_fields.exports.json_value;
    if !exports.is_present() {
        return false;
    }
    if exports.value_type != JsonValueType::Object {
        return true;
    }
    let package_name = fields
        .header_fields
        .name
        .get_value()
        .cloned()
        .unwrap_or_default();
    let subpath = fragment
        .strip_prefix(&package_name)
        .unwrap_or(fragment)
        .trim_start_matches('/')
        .to_string();
    for (key, value) in exports.as_object() {
        if key == "." {
            continue;
        }
        let normalized_key = key.trim_start_matches("./");
        for pattern in patterns_of_condition(value) {
            for name in completions_for_path_mapping(
                service,
                program,
                normalized_key,
                &[pattern],
                &subpath,
                package_dir,
            ) {
                result.add(name);
            }
        }
    }
    true
}

/// Go getPatternFromFirstMatchingCondition：default/types/import 或首个子键
fn patterns_of_condition(value: &JsonValue) -> Vec<String> {
    if value.value_type == JsonValueType::String {
        return vec![value.as_string().to_string()];
    }
    if value.value_type != JsonValueType::Object {
        return Vec::new();
    }
    for key in ["default", "types", "import"] {
        if let Some(v) = value.get(key)
            && v.value_type == JsonValueType::String
        {
            return vec![v.as_string().to_string()];
        }
    }
    value
        .as_object()
        .iter()
        .find(|(_, v)| v.value_type == JsonValueType::String)
        .map(|(_, v)| vec![v.as_string().to_string()])
        .unwrap_or_default()
}

/// Go getCompletionEntriesForDirectoryFragment（非相对形态）：
/// typesVersions 版本重定向命中即阻断普通目录枚举
fn directory_fragment_entries(
    service: &LanguageService,
    program: &Arc<Program>,
    fragment: &str,
    node_modules: &str,
    result: &mut ModuleCompletionSet,
) {
    let mut frag = fragment.to_string();
    if !tsp::has_trailing_directory_separator(&frag) {
        frag = tsp::get_directory_path(&frag);
    }
    if frag.is_empty() {
        frag = ".".to_string();
    }
    frag = tsp::ensure_trailing_directory_separator(&frag);
    let base_directory = normalize_join(node_modules, &frag);

    if version_redirect(service, program, &base_directory, result) {
        return;
    }
    if !service.directory_exists(&base_directory) {
        return;
    }
    let extensions = string_extensions(program);
    for file_path in service.read_directory(&base_directory, &extensions, &["./*".to_string()]) {
        result.add(completion_file_name(&tsp::get_base_file_name(&file_path)));
    }
    for dir in service.get_directories(&base_directory) {
        let name = tsp::get_base_file_name(&dir);
        if name != "@types" {
            result.add(name);
        }
    }
}

pub(super) fn get_fragment_directory(fragment: &str) -> String {
    if !fragment.contains('/') {
        return String::new();
    }
    if tsp::has_trailing_directory_separator(fragment) {
        fragment.to_string()
    } else {
        tsp::get_directory_path(fragment)
    }
}

fn module_resolution_uses_node_modules(program: &Arc<Program>) -> bool {
    matches!(
        program.options().get_module_resolution_kind(),
        ModuleResolutionKind::Node16 | ModuleResolutionKind::NodeNext | ModuleResolutionKind::Bundler
    )
}

pub(super) fn string_extensions(program: &Arc<Program>) -> Vec<String> {
    let mut exts: Vec<String> = Vec::new();
    // Go getSupportedExtensionsForModuleResolution：环境模块 `*.ext` 声明的
    // 扩展名（declare module "*.ruhroh"）先入列
    for name in ambient_module_names(program) {
        if let Some(ext) = name.strip_prefix("*.") {
            if !ext.contains('/') && !exts.iter().any(|e| e == &name[1..]) {
                exts.push(name[1..].to_string());
            }
        }
    }
    for ext in [
        ".d.ts", ".ts", ".tsx", ".js", ".jsx", ".mts", ".cts", ".mjs", ".cjs",
    ] {
        if !exts.iter().any(|e| e == ext) {
            exts.push(ext.to_string());
        }
    }
    if program.options().get_resolve_json_module() && !exts.iter().any(|e| e == ".json") {
        exts.push(".json".to_string());
    }
    exts
}

/// 默认 ImportModuleSpecifierEnding=minimal：JS/TS 实现扩展名剥离
pub(super) fn completion_file_name(name: &str) -> String {
    for ext in [
        ".d.ts", ".ts", ".tsx", ".js", ".jsx", ".mts", ".cts", ".mjs", ".cjs",
    ] {
        if let Some(stripped) = name.strip_suffix(ext) {
            return stripped.to_string();
        }
    }
    name.to_string()
}

pub(super) fn read_package_json(service: &LanguageService, package_dir: &str) -> Option<packagejson::Fields> {
    let path = tsp::combine_paths(package_dir, &["package.json"]);
    let content = service.read_file(&path)?;
    packagejson::parse(&content).ok()
}

fn normalize_join(base: &str, fragment: &str) -> String {
    tsp::normalize_path(&tsp::combine_paths(base, &[fragment]))
}

pub(super) fn nearest_package_json_dir(service: &LanguageService, from: &str) -> Option<String> {
    let mut dir = from.trim_end_matches('/').to_string();
    loop {
        if service
            .read_file(&tsp::combine_paths(&dir, &["package.json"]))
            .is_some()
        {
            return Some(dir);
        }
        let parent = tsp::get_directory_path(&dir);
        if parent == dir || dir.is_empty() {
            return None;
        }
        dir = parent;
    }
}

/// 程序内环境模块声明名（`declare module "name"`）
fn ambient_module_names(program: &Arc<Program>) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for file in program.source_files() {
        let is_dts = file.file_name.ends_with(".d.ts");
        let NodeData::SourceFile(sf) = &file.node.data else {
            continue;
        };
        for stmt in sf.statements.iter() {
            let NodeData::ModuleDeclaration(md) = &stmt.data else {
                continue;
            };
            if md.name.kind != SyntaxKind::StringLiteral {
                continue;
            }
            let declared = stmt.syntactic_modifier_flags().contains(ModifierFlags::Ambient);
            if !declared && !is_dts {
                continue;
            }
            let name = md.name.text().trim_matches(['"', '\'']).to_string();
            if !name.is_empty() && !names.contains(&name) {
                names.push(name);
            }
        }
    }
    names
}

/// Go UnmangleScopedPackageName：`@types/a__b` 目录名还原为 `@a/b`
fn unmangle_scoped_package_name(dir_name: &str) -> String {
    match dir_name.find("__") {
        Some(idx) => format!("@{}/{}", &dir_name[..idx], &dir_name[idx + 2..]),
        None => dir_name.to_string(),
    }
}

/// Go exportsOrImportsLookup（isImports）：package.json imports 表的
/// 模式枚举，返回是否被 imports 接管
fn imports_lookup(
    service: &LanguageService,
    program: &Arc<Program>,
    fragment: &str,
    directory: &str,
    result: &mut ModuleCompletionSet,
) -> bool {
    let Some(fields) = read_package_json(service, directory) else {
        return false;
    };
    let imports = &fields.path_fields.imports.json_value;
    if !imports.is_present() {
        return false;
    }
    if imports.value_type != JsonValueType::Object {
        return true;
    }
    for (key, value) in imports.as_object() {
        if !key.starts_with('#') {
            continue;
        }
        for pattern in patterns_of_condition(value) {
            for name in completions_for_path_mapping(
                service,
                program,
                key,
                &[pattern],
                fragment,
                directory,
            ) {
                result.add(name);
            }
        }
    }
    true
}
