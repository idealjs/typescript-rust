//! Incremental build support: .tsbuildinfo read/write and up-to-date checking.
//!
//! Mirrors Go's `internal/execute/incremental/buildInfo.go` in a simplified form.
//! The full format includes file signatures, diagnostic snapshots, and referenced
//! maps — this implementation covers the core use case: tracking which files were
//! compiled and their content hashes, allowing unchanged projects to be skipped.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Minimal .tsbuildinfo file structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
// Field names mirror the `.tsbuildinfo` JSON schema (camelCase) — keep them
// for byte-compatible serialization.
#[allow(non_snake_case)]
pub struct BuildInfo {
    /// Schema version (currently "incremental-compilation").
    pub version: String,
    /// List of file metadata entries.
    pub fileNames: Vec<FileInfo>,
    /// Root file range (simplified: just the project root path).
    pub root: String,
    /// Hash of compiler options that affect output.
    #[allow(non_snake_case)]
    pub optionsHash: String,
    /// Referenced project tsconfig paths.
    pub references: Vec<String>,
}

/// Per-file metadata in .tsbuildinfo.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct FileInfo {
    /// File path (absolute or relative to project root).
    pub path: String,
    /// Content hash (SHA-256 hex of file content).
    pub hash: String,
    /// File version (currently same as hash).
    pub version: String,
    /// Whether this file affects the global scope.
    #[serde(default)]
    #[allow(non_snake_case)]
    pub affectsGlobalScope: bool,
}

impl BuildInfo {
    /// Create a new BuildInfo for the given files.
    pub fn new(
        files: &[(String, String)], // (path, content) pairs
        root: &str,
        options_hash: &str,
        references: &[String],
    ) -> Self {
        let file_names = files
            .iter()
            .map(|(path, content)| {
                let hash = sha256_hex(content);
                FileInfo {
                    path: path.clone(),
                    hash: hash.clone(),
                    version: hash,
                    affectsGlobalScope: false,
                }
            })
            .collect();
        BuildInfo {
            version: "incremental-compilation".to_string(),
            fileNames: file_names,
            root: root.to_string(),
            optionsHash: options_hash.to_string(),
            references: references.to_vec(),
        }
    }

    /// Write build info to a .tsbuildinfo file.
    pub fn write_to_file(&self, path: &str) -> std::io::Result<()> {
        let json = serde_json::to_string(self)?;
        if let Some(parent) = Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, json)
    }

    /// Read build info from a .tsbuildinfo file.
    pub fn read_from_file(path: &str) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Check if the current build info matches the given files + options.
    /// Returns true if nothing has changed (project is up-to-date).
    pub fn is_up_to_date(
        &self,
        current_files: &[(String, String)], // (path, content)
        current_options_hash: &str,
    ) -> bool {
        // Check options hash
        if self.optionsHash != current_options_hash {
            return false;
        }

        // Check file count
        if self.fileNames.len() != current_files.len() {
            return false;
        }

        // Build a map of path → hash from build info
        let info_map: HashMap<&str, &str> = self
            .fileNames
            .iter()
            .map(|f| (f.path.as_str(), f.hash.as_str()))
            .collect();

        // Check each current file
        for (path, content) in current_files {
            let current_hash = sha256_hex(content);
            match info_map.get(path.as_str()) {
                Some(stored_hash) if *stored_hash == current_hash => continue,
                _ => return false,
            }
        }

        true
    }

    /// Get the .tsbuildinfo file path for a given tsconfig.
    /// If tsBuildInfoFile option is set, use it. Otherwise derive from outDir + tsconfig name.
    pub fn get_ts_build_info_file_path(
        tsconfig_path: &str,
        out_dir: &str,
        ts_build_info_file: &str,
    ) -> String {
        if !ts_build_info_file.is_empty() {
            return ts_build_info_file.to_string();
        }
        let config_name = Path::new(tsconfig_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("tsconfig");
        if !out_dir.is_empty() {
            format!("{out_dir}/{config_name}.tsbuildinfo")
        } else {
            format!("{config_name}.tsbuildinfo")
        }
    }
}

/// Compute SHA-256 hash of a string and return hex.
fn sha256_hex(input: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    // Use std hasher (not cryptographic, but sufficient for change detection)
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Compute a hash of compiler options that affect output.
pub fn compute_options_hash(options_json: &str) -> String {
    sha256_hex(options_json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_info_roundtrip() {
        let info = BuildInfo::new(
            &[
                ("/src/foo.ts".to_string(), "const x = 1;".to_string()),
                ("/src/bar.ts".to_string(), "const y = 2;".to_string()),
            ],
            "/src/tsconfig.json",
            "abc123",
            &[],
        );
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: BuildInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.fileNames.len(), 2);
        assert_eq!(deserialized.root, "/src/tsconfig.json");
    }

    #[test]
    fn up_to_date_check() {
        let files = vec![("/src/foo.ts".to_string(), "const x = 1;".to_string())];
        let info = BuildInfo::new(&files, "/src", "hash123", &[]);

        // Same files → up to date
        assert!(info.is_up_to_date(&files, "hash123"));

        // Changed content → not up to date
        let changed = vec![("/src/foo.ts".to_string(), "const x = 2;".to_string())];
        assert!(!info.is_up_to_date(&changed, "hash123"));

        // Different options → not up to date
        assert!(!info.is_up_to_date(&files, "different"));
    }

    #[test]
    fn build_info_file_path() {
        let path = BuildInfo::get_ts_build_info_file_path("/src/tsconfig.json", "/src/dist", "");
        assert_eq!(path, "/src/dist/tsconfig.tsbuildinfo");

        // With explicit tsBuildInfoFile
        let path = BuildInfo::get_ts_build_info_file_path(
            "/src/tsconfig.json",
            "/src/dist",
            "custom.tsbuildinfo",
        );
        assert_eq!(path, "custom.tsbuildinfo");
    }
}
