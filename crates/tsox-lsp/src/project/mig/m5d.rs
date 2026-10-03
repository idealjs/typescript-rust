use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tsox_core::tspath::Path;
use tsox_tsoptions::vfs::FS;

use crate::project::logging_log_tree::LogTree;
use crate::project::overlay_fs::{DiskFile, FileContent, FileHandle, Overlay, OverlayFS};

static M5D_SEQ: AtomicU64 = AtomicU64::new(0);

pub struct MigLogEntry {
    pub seq: u64,
    pub time: String,
    pub message: String,
    pub child: Option<Box<LogTree>>,
}

pub fn new_log_entry(child: Option<Box<LogTree>>, message: &str) -> MigLogEntry { ::tsox_core::fntrace::enter("new_log_entry"); 
    MigLogEntry {
        seq: M5D_SEQ.fetch_add(1, Ordering::SeqCst),
        time: format_time_now(),
        message: message.to_string(),
        child,
    }
}

fn format_time_now() -> String { ::tsox_core::fntrace::enter("format_time_now"); 
    "[time]".to_string()
}

pub trait LogTreeMigExt {
    fn error(&self, msg: &str);
    fn errorf(&self, format: &str, args: &[&dyn std::fmt::Display]);
    fn info(&self, msg: &str);
    fn infof(&self, format: &str, args: &[&dyn std::fmt::Display]);
    fn warn(&self, msg: &str);
    fn warnf(&self, format: &str, args: &[&dyn std::fmt::Display]);
    fn verbose(&self) -> Option<&LogTree>;
    fn string(&self) -> String;
}

impl LogTreeMigExt for LogTree {
    fn error(&self, msg: &str) { ::tsox_core::fntrace::enter("error"); 
        self.log(msg);
    }

    fn errorf(&self, format: &str, args: &[&dyn std::fmt::Display]) { ::tsox_core::fntrace::enter("errorf"); 
        self.logf(format, args);
    }

    fn info(&self, msg: &str) { ::tsox_core::fntrace::enter("info"); 
        self.log(msg);
    }

    fn infof(&self, format: &str, args: &[&dyn std::fmt::Display]) { ::tsox_core::fntrace::enter("infof"); 
        self.logf(format, args);
    }

    fn warn(&self, msg: &str) { ::tsox_core::fntrace::enter("warn"); 
        self.log(msg);
    }

    fn warnf(&self, format: &str, args: &[&dyn std::fmt::Display]) { ::tsox_core::fntrace::enter("warnf"); 
        self.logf(format, args);
    }

    fn verbose(&self) -> Option<&LogTree> { ::tsox_core::fntrace::enter("verbose"); 
        if !self.is_verbose() {
            return None;
        }
        Some(self)
    }

    fn string(&self) -> String { ::tsox_core::fntrace::enter("string"); 
        self.to_string()
    }
}

impl DiskFile {
    pub fn clone(&self) -> DiskFile { ::tsox_core::fntrace::enter("clone"); 
        let mut cloned = DiskFile::new(
            self.file_name().to_string(),
            self.content().to_string(),
        );
        cloned.realpath_path = self.realpath_path.clone();
        cloned
    }
}

impl crate::project::overlay_fs::FileBase {
    pub fn lsp_line_map(&self) -> Arc<crate::ls::lsconv_linemap::LspLineMap> { ::tsox_core::fntrace::enter("lsp_line_map"); 
        let content = self.content();
        Arc::new(crate::ls::lsconv_linemap::compute_lsp_line_starts(content))
    }

    pub fn ecma_line_info(&self) -> Arc<crate::mig::m6b::EcmaLineInfo> { ::tsox_core::fntrace::enter("ecma_line_info"); 
        let content = self.content();
        let line_starts = tsox_core::core::mig::m3j::compute_ecma_line_starts(content);
        Arc::new(crate::mig::m6b::create_ecma_line_info(
            content.to_string(),
            line_starts,
        ))
    }
}

impl Overlay {
    pub fn original_file_name(&self) -> &str { ::tsox_core::fntrace::enter("original_file_name"); 
        self.file_name()
    }

    pub fn span_map(&self) -> Option<&crate::mig::m6b_2::SpanMap> { ::tsox_core::fntrace::enter("span_map"); 
        None
    }

    pub fn original_text(&self) -> &str { ::tsox_core::fntrace::enter("original_text"); 
        self.content()
    }

    pub fn compute_matches_disk_text(&self, fs: &dyn FS) -> (bool, bool) { ::tsox_core::fntrace::enter("compute_matches_disk_text"); 
        let file_name = self.file_name();
        if tsox_core::tspath::is_dynamic_file_name(file_name) {
            return (false, false);
        }
        let disk_content = match fs.read_file(file_name) {
            Some(content) => content,
            None => return (false, false),
        };
        (
            crate::project::overlay_fs::hash_string_128(&disk_content) == self.hash(),
            true,
        )
    }
}

pub fn new_disk_file(file_name: String, content: String) -> DiskFile { ::tsox_core::fntrace::enter("new_disk_file"); 
    DiskFile::new(file_name, content)
}

pub fn new_overlay(file_name: String, content: String, version: i32, kind: i32) -> Overlay { ::tsox_core::fntrace::enter("new_overlay"); 
    Overlay::new(file_name, content, version, kind)
}

pub fn new_overlay_fs(
    fs: Arc<dyn FS>,
    overlays: std::collections::HashMap<Path, Arc<Overlay>>,
    position_encoding: crate::lsp::lsproto::PositionEncodingKind,
    to_path: Box<dyn Fn(&str) -> Path + Send + Sync>,
) -> OverlayFS { ::tsox_core::fntrace::enter("new_overlay_fs"); 
    OverlayFS::new(fs, overlays, position_encoding, to_path)
}
