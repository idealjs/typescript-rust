#![allow(unused_imports, dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use tsox_frontend::ast::diagnostic::Diagnostic;

use tsox_core::tspath;

use super::m5j::*;
use super::m5j_2::*;
use crate::tsoptions::build_options::ParsedCommandLine as CrateParsedCommandLine;
use crate::vfs::fs::FS;
use crate::vfs::types::{Entries, FileInfo};

pub struct VfsParseConfigHost {
    pub vfs: Arc<dyn FS>,
    pub current_directory: String,
}

impl VfsParseConfigHost {
    pub fn fs(&self) -> &dyn FS { ::tsox_core::fntrace::enter("fs"); 
        self.vfs.as_ref()
    }

    pub fn get_current_directory(&self) -> &str { ::tsox_core::fntrace::enter("get_current_directory"); 
        &self.current_directory
    }
}

pub fn new_vfs_parse_config_host(
    files: &HashMap<String, String>,
    current_directory: &str,
    use_case_sensitive_file_names: bool,
) -> VfsParseConfigHost { ::tsox_core::fntrace::enter("new_vfs_parse_config_host"); 
    VfsParseConfigHost {
        vfs: vfstest_from_map(files, use_case_sensitive_file_names),
        current_directory: current_directory.to_string(),
    }
}

pub fn new_vfs_parse_config_host_with_symlinks(
    files: &HashMap<String, String>,
    symlinks: &HashMap<String, String>,
    current_directory: &str,
    use_case_sensitive_file_names: bool,
) -> VfsParseConfigHost { ::tsox_core::fntrace::enter("new_vfs_parse_config_host_with_symlinks"); 
    if symlinks.is_empty() {
        return new_vfs_parse_config_host(files, current_directory, use_case_sensitive_file_names);
    }
    let mut entries: HashMap<String, VfstestEntry> = HashMap::new();
    for (name, content) in files {
        entries.insert(name.clone(), VfstestEntry::Content(content.clone()));
    }
    for (link, target) in symlinks {
        entries.insert(
            tspath::get_normalized_absolute_path(link, current_directory),
            VfstestEntry::Symlink(tspath::get_normalized_absolute_path(target, current_directory)),
        );
    }
    VfsParseConfigHost {
        vfs: vfstest_from_map_entries(&entries, use_case_sensitive_file_names),
        current_directory: current_directory.to_string(),
    }
}

pub enum VfstestEntry {
    Content(String),
    Symlink(String),
}

pub fn vfstest_from_map(files: &HashMap<String, String>, use_case_sensitive_file_names: bool) -> Arc<dyn FS> { ::tsox_core::fntrace::enter("vfstest_from_map"); 
    let _ = (files, use_case_sensitive_file_names);
    Arc::new(NoopFS)
}

pub fn vfstest_from_map_entries(entries: &HashMap<String, VfstestEntry>, use_case_sensitive_file_names: bool) -> Arc<dyn FS> { ::tsox_core::fntrace::enter("vfstest_from_map_entries"); 
    let _ = (entries, use_case_sensitive_file_names);
    Arc::new(NoopFS)
}

struct NoopFS;

impl FS for NoopFS {
    fn use_case_sensitive_file_names(&self) -> bool { ::tsox_core::fntrace::enter("use_case_sensitive_file_names"); 
        true
    }
    fn file_exists(&self, _path: &str) -> bool { ::tsox_core::fntrace::enter("file_exists"); 
        false
    }
    fn read_file(&self, _path: &str) -> Option<String> { ::tsox_core::fntrace::enter("read_file"); 
        None
    }
    fn write_file(&self, _path: &str, _data: &str) -> std::io::Result<()> { ::tsox_core::fntrace::enter("write_file"); 
        Ok(())
    }
    fn append_file(&self, _path: &str, _data: &str) -> std::io::Result<()> { ::tsox_core::fntrace::enter("append_file"); 
        Ok(())
    }
    fn remove(&self, _path: &str) -> std::io::Result<()> { ::tsox_core::fntrace::enter("remove"); 
        Ok(())
    }
    fn directory_exists(&self, _path: &str) -> bool { ::tsox_core::fntrace::enter("directory_exists"); 
        false
    }
    fn get_accessible_entries(&self, _path: &str) -> Entries { ::tsox_core::fntrace::enter("get_accessible_entries"); 
        Entries::default()
    }
    fn stat(&self, _path: &str) -> Option<FileInfo> { ::tsox_core::fntrace::enter("stat"); 
        None
    }
    fn realpath(&self, path: &str) -> String { ::tsox_core::fntrace::enter("realpath"); 
        path.to_string()
    }
}

pub fn fix_root(path: &str) -> String { ::tsox_core::fntrace::enter("fix_root"); 
    let root_length = tspath::get_root_length(path);
    if root_length == 0 {
        return path.to_string();
    }
    if path.len() == root_length {
        return ".".to_string();
    }
    path[root_length..].to_string()
}

pub fn get_parsed_command_line(
    json_text: &str,
    files: &HashMap<String, String>,
    current_directory: &str,
    use_case_sensitive_file_names: bool,
) -> CrateParsedCommandLine { ::tsox_core::fntrace::enter("get_parsed_command_line"); 
    let host = new_vfs_parse_config_host(files, current_directory, use_case_sensitive_file_names);
    let config_file_name = tspath::combine_paths(current_directory, &["tsconfig.json"]);
    let config_path = tspath::to_path(&config_file_name, current_directory, use_case_sensitive_file_names);
    let (source_file, _errors) = read_json_config_file(&config_file_name, config_path.as_str(), &|_| {
        Some(json_text.to_string())
    });
    parse_json_source_file_config_file_content(
        &source_file,
        &ParseConfigHost {
            fs: host.vfs.clone(),
            current_directory: host.current_directory.clone(),
        },
        current_directory,
        None,
        None,
        &config_file_name,
        &[],
        None,
    )
}

pub fn parse_json_source_file_config_file_content(
    tsconfig_source_file: &TsConfigSourceFile,
    host: &ParseConfigHost,
    base_path: &str,
    existing_options: Option<&tsox_core::core::compiler_options::CompilerOptions>,
    existing_options_raw: Option<&JsonObject>,
    config_file_name: &str,
    resolution_stack: &[String],
    extended_config_cache: Option<&ExtendedConfigCacheValue>,
) -> CrateParsedCommandLine { ::tsox_core::fntrace::enter("parse_json_source_file_config_file_content"); 
    let _ = (existing_options, existing_options_raw, extended_config_cache);
    parse_json_config_file_content_worker(
        None,
        Some(&mut {
            TsConfigSourceFile {
                extended_source_files: tsconfig_source_file.extended_source_files.clone(),
                config_file_specs: tsconfig_source_file.config_file_specs.clone(),
                source_file: tsconfig_source_file.source_file.clone(),
            }
        }),
        host,
        base_path,
        existing_options,
        existing_options_raw,
        config_file_name,
        resolution_stack,
        extended_config_cache,
    )
}

pub fn get_wildcard_directories(
    include: &[String],
    exclude: &[String],
    compare_paths_options: &tsox_core::tspath::ComparePathsOptions,
) -> HashMap<String, bool> { ::tsox_core::fntrace::enter("get_wildcard_directories"); 
    if include.is_empty() {
        return HashMap::new();
    }
    let exclude_matcher = vfsmatch_new_spec_matcher(
        exclude,
        &compare_paths_options.current_directory,
        vfsmatch::Usage::Exclude,
        compare_paths_options.use_case_sensitive_file_names,
    );
    let mut wildcard_directories: HashMap<String, bool> = HashMap::new();
    let mut wild_card_key_to_path: HashMap<String, String> = HashMap::new();
    let mut recursive_keys: Vec<String> = Vec::new();

    for file in include {
        let spec = tspath::normalize_path(&tspath::combine_paths(
            &compare_paths_options.current_directory,
            &[file.as_str()],
        ));
        if let Some(exclude_matcher) = &exclude_matcher {
            if exclude_matcher.match_string(&spec) {
                continue;
            }
        }
        if let Some(match_result) =
            get_wildcard_directory_from_spec(&spec, compare_paths_options.use_case_sensitive_file_names)
        {
            let key = match_result.key;
            let path = match_result.path;
            let recursive = match_result.recursive;

            let existing_path = wild_card_key_to_path.get(&key).cloned();
            let existing_recursive = existing_path
                .as_ref()
                .and_then(|p| wildcard_directories.get(p).copied())
                .unwrap_or(false);

            let should_set = match &existing_path {
                None => true,
                Some(_) => !existing_recursive && recursive,
            };
            if should_set {
                let path_to_use = existing_path.clone().unwrap_or_else(|| path.clone());
                wildcard_directories.insert(path_to_use, recursive);
                if existing_path.is_none() {
                    wild_card_key_to_path.insert(key.clone(), path.clone());
                }
                if recursive {
                    recursive_keys.push(key.clone());
                }
            }
        }

        let paths: Vec<String> = wildcard_directories.keys().cloned().collect();
        for path in paths {
            let key = to_canonical_key(&path, compare_paths_options.use_case_sensitive_file_names);
            for recursive_key in &recursive_keys {
                if key != *recursive_key
                    && tspath::Path::from(recursive_key.as_str())
                        .contains_path(&tspath::Path::from(path.as_str()))
                {
                    wildcard_directories.remove(&path);
                }
            }
        }
    }
    wildcard_directories
}

pub fn to_canonical_key(path: &str, use_case_sensitive_file_names: bool) -> String { ::tsox_core::fntrace::enter("to_canonical_key"); 
    if use_case_sensitive_file_names {
        path.to_string()
    } else {
        path.to_lowercase()
    }
}

pub struct WildcardDirectoryMatch {
    pub key: String,
    pub path: String,
    pub recursive: bool,
}

pub fn get_wildcard_directory_from_spec(spec: &str, use_case_sensitive_file_names: bool) -> Option<WildcardDirectoryMatch> { ::tsox_core::fntrace::enter("get_wildcard_directory_from_spec"); 
    if let Some(first_wildcard) = spec.find(|c| c == '*' || c == '?') {
        if let Some(last_sep_before_wildcard) = spec[..first_wildcard].rfind(tspath::DIRECTORY_SEPARATOR) {
            let path = &spec[..last_sep_before_wildcard];
            let last_directory_separator_index = spec.rfind(tspath::DIRECTORY_SEPARATOR).unwrap_or(0);
            let recursive = first_wildcard < last_directory_separator_index;
            return Some(WildcardDirectoryMatch {
                key: to_canonical_key(path, use_case_sensitive_file_names),
                path: path.to_string(),
                recursive,
            });
        }
    }
    if let Some(last_sep_index) = spec.rfind(tspath::DIRECTORY_SEPARATOR) {
        let last_segment = &spec[last_sep_index + 1..];
        if vfsmatch_is_implicit_glob(last_segment) {
            let path = tspath::remove_trailing_directory_separator(spec);
            return Some(WildcardDirectoryMatch {
                key: to_canonical_key(&path, use_case_sensitive_file_names),
                path,
                recursive: true,
            });
        }
    }
    None
}

pub fn vfsmatch_is_implicit_glob(_last_segment: &str) -> bool { ::tsox_core::fntrace::enter("vfsmatch_is_implicit_glob"); 
    false
}
