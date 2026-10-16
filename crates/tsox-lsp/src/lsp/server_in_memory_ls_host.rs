#![allow(dead_code)]

use serde_json::Value;

use crate::ls::host::{AutoImportRegistry, EcmaLineInfo, Host};
use crate::ls::lsconv_converters::Converters;
use crate::ls::lsconv_converters::PositionEncodingKind;
use crate::ls::lsutil::new_default_user_preferences;

use crate::lsp::server::*;

impl Host for InMemoryLsHost {
    fn use_case_sensitive_file_names(&self) -> bool { ::tsox_core::fntrace::enter("use_case_sensitive_file_names"); 
        self.case_sensitive
    }

    fn read_file(&self, _path: &str) -> Option<String> { ::tsox_core::fntrace::enter("read_file"); 
        None
    }

    fn converters(&self) -> Converters { ::tsox_core::fntrace::enter("converters"); 
        Converters::new(PositionEncodingKind::Utf16)
    }

    fn get_preferences(&self, _active_file: &str) -> crate::ls::lsutil::UserPreferences { ::tsox_core::fntrace::enter("get_preferences"); 
        new_default_user_preferences()
    }

    fn get_ecma_line_info(&self, _file_name: &str) -> Option<EcmaLineInfo> { ::tsox_core::fntrace::enter("get_ecma_line_info"); 
        None
    }

    fn auto_import_registry(&self) -> AutoImportRegistry { ::tsox_core::fntrace::enter("auto_import_registry"); 
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
    ) -> Vec<String> { ::tsox_core::fntrace::enter("read_directory"); 
        Vec::new()
    }

    fn get_directories(&self, _path: &str) -> Vec<String> { ::tsox_core::fntrace::enter("get_directories"); 
        Vec::new()
    }

    fn directory_exists(&self, _path: &str) -> bool { ::tsox_core::fntrace::enter("directory_exists"); 
        false
    }

    fn file_exists(&self, _path: &str) -> bool { ::tsox_core::fntrace::enter("file_exists"); 
        false
    }
}

pub fn send_client_request_fire_and_forget(server: &Server, method: &str, params: &Value) { ::tsox_core::fntrace::enter("send_client_request_fire_and_forget"); 
    server.send_client_request(method, params);
}

pub fn send_notification(server: &Server, method: &str, params: &Value) { ::tsox_core::fntrace::enter("send_notification"); 
    server.send_notification(method, params);
}
