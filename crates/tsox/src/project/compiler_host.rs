#![allow(dead_code)]

use std::sync::Arc;

use crate::compiler::CompilerHost;
use crate::tspath::Path;
use crate::vfs::FS;

#[derive(Clone)]
pub struct SessionOptions {
    pub current_directory: String,
    pub default_library_path: String,
    pub typings_location: String,
    pub position_encoding: crate::lsp::lsproto::PositionEncodingKind,
    pub watch_enabled: bool,
    pub logging_enabled: bool,
    pub telemetry_enabled: bool,
    pub push_diagnostics_enabled: bool,
    pub debounce_delay: std::time::Duration,
    pub locale: String,
}

impl Default for SessionOptions {
    fn default() -> Self {
        SessionOptions {
            current_directory: String::new(),
            default_library_path: String::new(),
            typings_location: String::new(),
            position_encoding: crate::lsp::lsproto::POSITION_ENCODING_UTF16.to_string(),
            watch_enabled: false,
            logging_enabled: false,
            telemetry_enabled: false,
            push_diagnostics_enabled: false,
            debounce_delay: std::time::Duration::from_millis(250),
            locale: "en".to_string(),
        }
    }
}

pub struct SessionInit {
    pub options: SessionOptions,
    pub fs: Arc<dyn FS>,
}

pub struct CompilerHostImpl {
    config_file_path: Path,
    current_directory: String,
    session_options: SessionOptions,
    fs: Arc<dyn FS>,
    frozen: bool,
}

impl CompilerHostImpl {
    pub fn new(
        current_directory: String,
        project_path: Path,
        session_options: SessionOptions,
        fs: Arc<dyn FS>,
    ) -> Self {
        CompilerHostImpl {
            config_file_path: project_path,
            current_directory,
            session_options,
            fs,
            frozen: false,
        }
    }

    pub fn freeze(&mut self) {
        self.frozen = true;
    }

    pub fn ensure_alive(&self) {
        if self.frozen {
            panic!("method must not be called after snapshot initialization");
        }
    }
}

impl CompilerHost for CompilerHostImpl {
    fn fs(&self) -> &dyn FS {
        self.fs.as_ref()
    }
    fn fs_arc(&self) -> Arc<dyn FS> {
        Arc::clone(&self.fs)
    }
    fn current_directory(&self) -> &str {
        &self.current_directory
    }
    fn default_library_path(&self) -> &str {
        &self.session_options.default_library_path
    }
}
