#![allow(dead_code)]

use crate::ls::lsconv::converters::Converters;
use crate::ls::lsutil::UserPreferences;

pub struct EcmaLineInfo;

pub struct AutoImportRegistry;

pub trait Host: Send + Sync {
    fn use_case_sensitive_file_names(&self) -> bool;
    fn read_file(&self, path: &str) -> Option<String>;
    fn converters(&self) -> Converters;
    fn get_preferences(&self, active_file: &str) -> UserPreferences;
    fn get_ecma_line_info(&self, file_name: &str) -> Option<EcmaLineInfo>;
    fn auto_import_registry(&self) -> AutoImportRegistry;

    fn read_directory(
        &self,
        current_dir: &str,
        path: &str,
        extensions: &[String],
        excludes: &[String],
        includes: &[String],
        depth: i32,
    ) -> Vec<String>;
    fn get_directories(&self, path: &str) -> Vec<String>;
    fn directory_exists(&self, path: &str) -> bool;
    fn file_exists(&self, path: &str) -> bool;
}
