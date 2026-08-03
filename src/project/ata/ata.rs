//! Automatic Type Acquisition (1:1 port of Go's `internal/project/ata/ata.go`).

use std::collections::HashMap;
use std::sync::atomic::AtomicI32;
use std::sync::{Arc, Mutex, Once};

use crate::collections::set::Set;
use crate::collections::syncmap::SyncMap;
use crate::core::compiler_options::CompilerOptions;
use crate::module;
use crate::semver;
use crate::tspath;
use crate::vfs::FS;

use super::discover_typings::{AtaLogger, CachedTyping, TypingsInfo};
use super::validate_package_name::{NameValidationResult, validate_package_name};

/// Options for the typings installer.
///
/// Mirrors `ata.TypingsInstallerOptions` in Go.
#[derive(Debug, Clone, Default)]
pub struct TypingsInstallerOptions {
    pub typings_location: String,
    pub throttle_limit: usize,
}

/// An NPM executor interface.
///
/// Mirrors `ata.NpmExecutor` interface in Go.
pub trait NpmExecutor: Send + Sync {
    fn npm_install(&self, cwd: &str, args: &[String]) -> Result<Vec<u8>, String>;
}

/// The typings installer host interface.
///
/// Mirrors `ata.TypingsInstallerHost` interface in Go.
pub trait TypingsInstallerHost: NpmExecutor + Send + Sync {
    fn get_current_directory(&self) -> &str;
    fn fs(&self) -> &dyn FS;
}

/// The version to use when requesting typings (Go uses "latest").
pub const TS_VERSION_TO_USE: &str = "latest";

/// A request to install typings.
///
/// Mirrors `ata.TypingsInstallRequest` in Go.
#[derive(Clone, Default)]
pub struct TypingsInstallRequest {
    pub project_id: tspath::Path,
    pub typings_info: TypingsInfo,
    pub file_names: Vec<String>,
    pub project_root_path: String,
    pub compiler_options: CompilerOptions,
    pub current_directory: String,
    pub fs: Option<Arc<dyn FS>>,
}

/// The result of installing typings.
///
/// Mirrors `ata.TypingsInstallResult` in Go.
#[derive(Debug, Clone, Default)]
pub struct TypingsInstallResult {
    pub typings_files: Vec<String>,
    pub files_to_watch: Vec<String>,
}

/// The typings installer.
///
/// Mirrors `ata.TypingsInstaller` in Go.
pub struct TypingsInstaller {
    pub typings_location: String,
    pub host: Arc<dyn TypingsInstallerHost>,

    init_once: Once,
    package_name_to_typing_location: SyncMap<String, Arc<CachedTyping>>,
    missing_typings_set: SyncMap<String, bool>,
    types_registry: Mutex<HashMap<String, HashMap<String, String>>>,
    install_run_count: AtomicI32,
}

impl TypingsInstaller {
    /// Creates a new typings installer.
    ///
    /// Mirrors `ata.NewTypingsInstaller` in Go.
    pub fn new(
        options: &TypingsInstallerOptions,
        host: Arc<dyn TypingsInstallerHost>,
    ) -> TypingsInstaller {
        TypingsInstaller {
            typings_location: options.typings_location.clone(),
            host,
            init_once: Once::new(),
            package_name_to_typing_location: SyncMap::new(),
            missing_typings_set: SyncMap::new(),
            types_registry: Mutex::new(HashMap::new()),
            install_run_count: AtomicI32::new(0),
        }
    }

    /// Checks if a package name is a known types package.
    ///
    /// Mirrors `(ti *TypingsInstaller) IsKnownTypesPackageName` in Go.
    pub fn is_known_types_package_name(
        &self,
        _project_id: &tspath::Path,
        name: &str,
        _fs: &dyn FS,
        _logger: Option<&dyn AtaLogger>,
    ) -> bool {
        let (validation_result, _, _) = validate_package_name(name);
        if validation_result != NameValidationResult::NameOk {
            return false;
        }
        let registry = self.types_registry.lock().unwrap();
        registry.contains_key(name)
    }

    /// Installs typings for a request.
    ///
    /// Mirrors `(ti *TypingsInstaller) InstallTypings` in Go.
    pub fn install_typings(
        &mut self,
        _request: &TypingsInstallRequest,
    ) -> Result<TypingsInstallResult, String> {
        // Requires full ATA infrastructure (npm install, FS, types registry) — stubbed.
        todo!("install_typings requires npm install and types registry infrastructure")
    }

    /// Initializes the typings installer (loads types registry, processes cache).
    ///
    /// Mirrors `(ti *TypingsInstaller) init` in Go.
    pub fn init(&self, _project_id: &str, _fs: &dyn FS, _logger: Option<&dyn AtaLogger>) {
        self.init_once.call_once(|| {
            // Full initialization requires npm install for types-registry,
            // FS access for cache processing — stubbed.
        });
    }

    /// Filters typings to only those that need installation.
    ///
    /// Mirrors `(ti *TypingsInstaller) filterTypings` in Go.
    pub fn filter_typings(
        &self,
        _project_id: &tspath::Path,
        _logger: Option<&dyn AtaLogger>,
        typings_to_install: &[String],
    ) -> Vec<String> {
        let mut result = Vec::new();
        for typing in typings_to_install {
            let typing_key = module::mangle_scoped_package_name(typing);
            if self.missing_typings_set.load(&typing_key).is_some() {
                continue;
            }
            let (validation_result, _name, _is_scope) = validate_package_name(typing);
            if validation_result != NameValidationResult::NameOk {
                self.missing_typings_set.store(typing_key.clone(), true);
                continue;
            }
            let registry = self.types_registry.lock().unwrap();
            if !registry.contains_key(&typing_key) {
                continue;
            }
            drop(registry);
            if let Some(_typing_location) = self.package_name_to_typing_location.load(&typing_key) {
                // Check if up to date — stubbed.
                continue;
            }
            result.push(typing_key);
        }
        result
    }

    /// Resolves a typing to a file name.
    ///
    /// Mirrors `(ti *TypingsInstaller) typingToFileName` in Go.
    pub fn typing_to_file_name(&self, _resolver: &module::Resolver, _package_name: &str) -> String {
        // Requires module resolver.ResolveModuleName — stubbed.
        todo!("typing_to_file_name requires module resolver")
    }
}

/// Installs npm packages in batches.
///
/// Mirrors `ata.installNpmPackages` in Go.
pub fn install_npm_packages(
    _package_names: &[String],
    _concurrency_limit: usize,
    _install_packages: &dyn Fn(&[String]) -> Result<(), String>,
) -> Result<(), String> {
    // The Go logic batches package names into groups of ~8000 chars and
    // installs them concurrently using a throttle group.
    // Stubbed until concurrency infrastructure is ported.
    todo!("install_npm_packages requires concurrency infrastructure")
}

/// NPM config structure (from package.json in typings cache).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct NpmConfig {
    #[serde(rename = "devDependencies", default)]
    pub dev_dependencies: HashMap<String, serde_json::Value>,
}

/// NPM lock file entry.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct NpmDependencyEntry {
    #[serde(default)]
    pub version: String,
}

/// NPM lock file structure (package-lock.json).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct NpmLock {
    #[serde(default)]
    pub dependencies: HashMap<String, NpmDependencyEntry>,
    #[serde(default)]
    pub packages: HashMap<String, NpmDependencyEntry>,
}
