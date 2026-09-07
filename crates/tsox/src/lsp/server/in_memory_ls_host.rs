#![allow(dead_code)]

use serde_json::Value;

use crate::ls::host::{AutoImportRegistry, EcmaLineInfo, Host};
use crate::ls::lsconv::converters::{Converters, PositionEncodingKind};
use crate::ls::lsutil::new_default_user_preferences;

use super::*;

impl Host for InMemoryLsHost {
    fn use_case_sensitive_file_names(&self) -> bool {
        self.case_sensitive
    }

    fn read_file(&self, _path: &str) -> Option<String> {
        None
    }

    fn converters(&self) -> Converters {
        Converters::new(PositionEncodingKind::Utf16)
    }

    fn get_preferences(&self, _active_file: &str) -> crate::ls::lsutil::UserPreferences {
        new_default_user_preferences()
    }

    fn get_ecma_line_info(&self, _file_name: &str) -> Option<EcmaLineInfo> {
        None
    }

    fn auto_import_registry(&self) -> AutoImportRegistry {
        AutoImportRegistry
    }

    fn read_directory(
        &self,
        _current_dir: &str,
        _path: &str,
        _extensions: &[String],
        _excludes: &[String],
        _includes: &[String],
        _depth: i32,
    ) -> Vec<String> {
        Vec::new()
    }

    fn get_directories(&self, _path: &str) -> Vec<String> {
        Vec::new()
    }

    fn directory_exists(&self, _path: &str) -> bool {
        false
    }

    fn file_exists(&self, _path: &str) -> bool {
        false
    }
}

pub fn send_client_request_fire_and_forget(server: &Server, method: &str, params: &Value) {
    server.send_client_request(method, params);
}

pub fn send_notification(server: &Server, method: &str, params: &Value) {
    server.send_notification(method, params);
}
