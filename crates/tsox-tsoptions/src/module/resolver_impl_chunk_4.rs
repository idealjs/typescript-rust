#![allow(unused_imports)]

use crate::module::resolver::*;

impl Resolver {
    pub fn new(
        host: Arc<dyn ResolutionHost + Send + Sync>,
        compiler_options: Arc<CompilerOptions>,
        typings_location: String,
        project_name: String,
    ) -> Self {
        Resolver {
            module_cache: ModuleResolutionCache::new(),
            type_ref_cache: TypeRefDirectiveResolutionCache::new(),
            host,
            compiler_options,
            typings_location,
            project_name,
        }
    }

    pub fn host(&self) -> &dyn ResolutionHost {
        self.host.as_ref()
    }

    pub fn compiler_options(&self) -> &CompilerOptions {
        &self.compiler_options
    }

    pub fn resolve_module_name(
        &self,
        module_name: &str,
        containing_file: &str,
        resolution_mode: ResolutionMode,
        _redirected_reference: Option<&str>,
    ) -> (Option<ResolvedModule>, Vec<DiagAndArgs>) {
        let containing_directory = tsox_core::tspath::get_directory_path(containing_file);
        let cache_key = ModuleResolutionCacheKey {
            containing_directory: containing_directory.to_string(),
            module_name: module_name.to_string(),
            resolution_mode,
            redirect_config_name: String::new(),
        };

        let trace = self.compiler_options.trace_resolution.is_true();
        if !trace {
            if let Some(cached) = self.module_cache.get(&cache_key) {
                return (Some((*cached).clone()), Vec::new());
            }
        }

        let effective_mode = default_resolution_mode(
            resolution_mode,
            &self.compiler_options,
            containing_file,
            self.host.fs(),
        );
        let mut state = ResolutionState::new(
            module_name,
            &containing_directory,
            false,
            effective_mode,
            &self.compiler_options,
            self.host.fs(),
            self.host.get_current_directory(),
        );
        let result = state.resolve_node_like();
        let diagnostics = std::mem::take(&mut state.resolution_diagnostics);
        let result_arc = Arc::new(result.clone());
        self.module_cache.set(cache_key, result_arc);
        (Some(result), diagnostics)
    }

    pub fn resolve_type_reference_directive(
        &self,
        type_reference_directive_name: &str,
        containing_file: &str,
        resolution_mode: ResolutionMode,
        _redirected_reference: Option<&str>,
    ) -> (Option<ResolvedTypeReferenceDirective>, Vec<DiagAndArgs>) {
        let containing_directory = tsox_core::tspath::get_directory_path(containing_file);
        let from_inferred_types_containing_file =
            containing_file.ends_with(crate::module::INFERRED_TYPES_CONTAINING_FILE);
        let cache_key = TypeRefDirectiveCacheKey {
            containing_directory: containing_directory.to_string(),
            type_reference_name: type_reference_directive_name.to_string(),
            resolution_mode,
            redirect_config_name: String::new(),
            from_inferred_types_containing_file,
        };

        let trace = self.compiler_options.trace_resolution.is_true();
        if !trace {
            if let Some(cached) = self.type_ref_cache.get(&cache_key) {
                return (Some((*cached).clone()), Vec::new());
            }
        }

        let fs = self.host.fs();
        let current_dir = self.host.get_current_directory();
        let (type_roots, from_config) =
            get_effective_type_roots(&self.compiler_options, current_dir);

        let mut state = ResolutionState::new(
            type_reference_directive_name,
            &containing_directory,
            true,
            default_resolution_mode(resolution_mode, &self.compiler_options, containing_file, fs),
            &self.compiler_options,
            fs,
            current_dir,
        );
        let result = state.resolve_type_reference_directive(
            &type_roots,
            from_config,
            from_inferred_types_containing_file,
        );

        let result_arc = Arc::new(result.clone());
        self.type_ref_cache.set(cache_key, result_arc);
        (Some(result), Vec::new())
    }
}

pub(crate) fn default_resolution_mode(
    resolution_mode: ResolutionMode,
    options: &CompilerOptions,
    containing_file: &str,
    fs: &dyn FS,
) -> ResolutionMode {
    if resolution_mode != ResolutionMode::None {
        return resolution_mode;
    }
    match options.get_module_resolution_kind() {
        ModuleResolutionKind::Node16 | ModuleResolutionKind::NodeNext => {
            crate::tsoptions::implied_node_format_of_file(containing_file, &|p| fs.read_file(p))
        }
        _ => ResolutionMode::None,
    }
}

pub fn get_effective_type_roots(
    options: &CompilerOptions,
    current_directory: &str,
) -> (Vec<String>, bool) {
    if !options.type_roots.is_empty() {
        return (options.type_roots.clone(), true);
    }
    let base_dir = if !options.config_file_path.is_empty() {
        tsox_core::tspath::get_directory_path(&options.config_file_path)
    } else {
        current_directory.to_string()
    };
    let mut type_roots = Vec::new();
    let mut dir = base_dir;
    loop {
        type_roots.push(tsox_core::tspath::combine_paths(
            &dir,
            &["node_modules", "@types"],
        ));
        let parent = tsox_core::tspath::get_directory_path(&dir);
        if parent == dir {
            break;
        }
        dir = parent;
    }
    (type_roots, false)
}

pub(crate) fn compute_package_id(fs: &dyn FS, resolved_file_name: &str) -> Option<PackageId> {
    if !resolved_file_name.contains("/node_modules/") {
        return None;
    }
    let package_directory = crate::module::parse_node_module_from_path(resolved_file_name, false);
    if package_directory.is_empty() {
        return None;
    }
    let pkg_json_path = tsox_core::tspath::combine_paths(&package_directory, &["package.json"]);
    let content = fs.read_file(&pkg_json_path)?;
    let fields = packagejson::parse(&content).ok()?;
    let name = fields.header_fields.name.get_value()?.clone();
    let version = fields.header_fields.version.get_value()?.clone();
    let sub_module_name = if resolved_file_name.len() > package_directory.len() {
        resolved_file_name[package_directory.len() + 1..].to_string()
    } else {
        String::new()
    };
    let peer_dependencies = read_peer_dependencies(fs, &fields, &package_directory);
    Some(PackageId {
        name,
        sub_module_name,
        version,
        peer_dependencies,
    })
}

fn read_peer_dependencies(
    fs: &dyn FS,
    fields: &packagejson::Fields,
    package_directory: &str,
) -> String {
    let Some(peers) = fields
        .dependency_fields
        .peer_dependencies
        .get_value()
        .filter(|p| !p.is_empty())
    else {
        return String::new();
    };
    let Some(idx) = package_directory.rfind("/node_modules") else {
        return String::new();
    };
    let node_modules = &package_directory[..idx + "/node_modules".len()];
    let mut names: Vec<&String> = peers.keys().collect();
    names.sort();
    let mut out = String::new();
    for name in names {
        let found = fs
            .read_file(&tsox_core::tspath::combine_paths(
                node_modules,
                &[name, "package.json"],
            ))
            .and_then(|c| packagejson::parse(&c).ok())
            .and_then(|f| f.header_fields.version.get_value().cloned());
        if let Some(version) = found {
            out.push('+');
            out.push_str(name);
            out.push('@');
            out.push_str(&version);
        }
    }
    out
}

pub(crate) struct ResolutionState<'a> {
    pub(crate) name: String,
    pub(crate) containing_directory: String,
    pub(crate) is_config_lookup: bool,
    pub(crate) features: NodeResolutionFeatures,
    pub(crate) esm_mode: bool,
    pub(crate) conditions: Vec<String>,
    pub(crate) extensions: Extensions,
    pub(crate) compiler_options: &'a CompilerOptions,
    #[allow(dead_code)]
    pub(crate) resolve_package_directory_only: bool,
    pub(crate) fs: &'a dyn FS,
    pub(crate) current_directory: &'a str,
    pub(crate) resolved_package_directory: bool,
    pub(crate) candidate_ending_is_from_config: bool,

    pub(crate) export_target_depth: u32,

    pub(crate) resolution_diagnostics: Vec<DiagAndArgs>,

    /// Go unresolved() 终止语义：exports/imports 目标反查已判定终止，
    /// 不再尝试后续条件/策略
    pub(crate) unresolved_terminal: bool,
}
