use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]

#[allow(non_snake_case)]
pub struct BuildInfo {

    pub version: String,

    pub fileNames: Vec<FileInfo>,

    pub root: String,

    #[allow(non_snake_case)]
    pub optionsHash: String,

    pub references: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct FileInfo {

    pub path: String,

    pub hash: String,

    pub version: String,

    #[serde(default)]
    #[allow(non_snake_case)]
    pub affectsGlobalScope: bool,
}

impl BuildInfo {

    pub fn new(
        files: &[(String, String)],
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

    pub fn write_to_file(&self, path: &str) -> std::io::Result<()> {
        let json = serde_json::to_string(self)?;
        if let Some(parent) = Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, json)
    }

    pub fn read_from_file(path: &str) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn is_up_to_date(
        &self,
        current_files: &[(String, String)],
        current_options_hash: &str,
    ) -> bool {

        if self.optionsHash != current_options_hash {
            return false;
        }

        if self.fileNames.len() != current_files.len() {
            return false;
        }

        let info_map: HashMap<&str, &str> = self
            .fileNames
            .iter()
            .map(|f| (f.path.as_str(), f.hash.as_str()))
            .collect();

        for (path, content) in current_files {
            let current_hash = sha256_hex(content);
            match info_map.get(path.as_str()) {
                Some(stored_hash) if *stored_hash == current_hash => continue,
                _ => return false,
            }
        }

        true
    }

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

fn sha256_hex(input: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

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

        assert!(info.is_up_to_date(&files, "hash123"));

        let changed = vec![("/src/foo.ts".to_string(), "const x = 2;".to_string())];
        assert!(!info.is_up_to_date(&changed, "hash123"));

        assert!(!info.is_up_to_date(&files, "different"));
    }

    #[test]
    fn build_info_file_path() {
        let path = BuildInfo::get_ts_build_info_file_path("/src/tsconfig.json", "/src/dist", "");
        assert_eq!(path, "/src/dist/tsconfig.tsbuildinfo");

        let path = BuildInfo::get_ts_build_info_file_path(
            "/src/tsconfig.json",
            "/src/dist",
            "custom.tsbuildinfo",
        );
        assert_eq!(path, "custom.tsbuildinfo");
    }
}
