//! fourslash 会话：解析结果 + 内存文件状态 + 真实 LanguageService。
//! 框架操作以自由函数形式提供（见 api 模块），Session 只承载数据。

use crate::fourslash::parse::{Marker, RangeMarker, TestData, parse_test_data};
use crate::ls::host::{AutoImportRegistry, EcmaLineInfo, Host};
use crate::ls::language_service::LanguageService;
use crate::ls::lsconv_converters::{Converters, PositionEncodingKind};
use crate::ls::lsutil::UserPreferences;
use std::sync::{Arc, Mutex};
use tsox_checker::bundled::{BundledFS, lib_path};
use tsox_compile::compiler::{CompilerHost, CompilerHostImpl, Program, ProgramOptions};
use tsox_core::tspath::Path;
use tsox_tsoptions::tsoptions::parse_command_line;
use tsox_tsoptions::vfs::{FS, InMemoryFS};

pub const DEFAULT_FILE_NAME: &str = "main.ts";

pub const PROJECT_ROOT: &str = "/";

/// 路径归一（对齐 path.Clean）：去掉 `.` 段、合并重复分隔符；`/./foo.ts` → `/foo.ts`
pub(crate) fn normalize_path(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for seg in path.split('/') {
        match seg {
            "" | "." => {}
            _ => parts.push(seg),
        }
    }
    let joined = parts.join("/");
    if path.starts_with('/') {
        format!("/{joined}")
    } else {
        joined
    }
}

pub(crate) fn project_path(name: &str) -> String {
    if name.starts_with('/') {
        normalize_path(name)
    } else {
        normalize_path(&format!("{PROJECT_ROOT}/{}", name))
    }
}

struct FourslashHost {
    fs: Arc<BundledFS>,
    prefs: Arc<Mutex<UserPreferences>>,
}

/// 虚拟 FS 的 ReadDirectory：按扩展名过滤文件，include glob 的
/// `./*`（当前层）与 `**/*后缀`（递归）两种形态
fn read_virtual_directory(
    fs: &dyn tsox_tsoptions::vfs::FS,
    path: &str,
    extensions: &[String],
    includes: &[String],
) -> Vec<String> {
    let recursive = includes.iter().any(|i| i.starts_with("**/"));
    let suffix_filters: Vec<&str> = includes
        .iter()
        .filter_map(|i| i.strip_prefix("**/*"))
        .collect();
    let mut out = Vec::new();
    let mut stack = vec![path.to_string()];
    while let Some(dir) = stack.pop() {
        let entries = fs.get_accessible_entries(&dir);
        for file in entries.files {
            let full = if dir.ends_with('/') {
                format!("{dir}{file}")
            } else {
                format!("{dir}/{file}")
            };
            let name_matches_ext = extensions.is_empty()
                || extensions.iter().any(|e| file.ends_with(e.as_str()));
            let name_matches_include = if recursive {
                suffix_filters.is_empty()
                    || suffix_filters.iter().any(|s| file.ends_with(s))
            } else {
                full.trim_start_matches(path).trim_start_matches('/').len()
                    == file.len()
            };
            if name_matches_ext && name_matches_include {
                out.push(full);
            }
        }
        if recursive {
            for sub in entries.directories {
                let full = if dir.ends_with('/') {
                    format!("{dir}{sub}")
                } else {
                    format!("{dir}/{sub}")
                };
                if !full.contains("/node_modules/") {
                    stack.push(full);
                }
            }
        }
    }
    out.sort();
    out
}

impl Host for FourslashHost {
    fn use_case_sensitive_file_names(&self) -> bool {
        false
    }
    fn read_file(&self, path: &str) -> Option<String> {
        self.fs.read_file(path)
    }
    fn converters(&self) -> Converters {
        Converters::new(PositionEncodingKind::Utf8)
    }
    fn get_preferences(&self, _active_file: &str) -> UserPreferences {
        self.prefs.lock().unwrap().clone()
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
        path: &str,
        extensions: &[String],
        _excludes: &[String],
        includes: &[String],
        _depth: i32,
    ) -> Vec<String> {
        read_virtual_directory(&*self.fs, path, extensions, includes)
    }
    fn get_directories(&self, path: &str) -> Vec<String> {
        self.fs.get_accessible_entries(path).directories
    }
    fn directory_exists(&self, path: &str) -> bool {
        self.fs.directory_exists(path)
    }
    fn file_exists(&self, path: &str) -> bool {
        self.fs.file_exists(path)
    }
}

pub struct Session {
    pub data: TestData,
    pub active_file: String,
    pub cursor: Option<usize>,
    /// Go fourslash stateEnableFormatting：输入/粘贴后触发格式化
    pub enable_formatting: bool,
    contents: std::collections::BTreeMap<String, String>,
    pub capabilities: Option<String>,
    inner_fs: Arc<InMemoryFS>,
    pub(crate) prefs: Arc<Mutex<UserPreferences>>,
    pub service: Option<LanguageService>,
}

impl Session {
    pub fn new(content: &str) -> Session {
        Self::new_impl(content, None, DEFAULT_FILE_NAME)
    }

    /// 按测试名命名默认文件（对齐 Go fourslash：defaultFileName = 测试名 + ".ts"）
    pub fn new_for_test(test_name: &str, content: &str) -> Session {
        Self::new_impl(content, None, &format!("{test_name}.ts"))
    }

    pub fn new_with_capabilities(content: &str, capabilities: Option<String>) -> Session {
        Self::new_impl(content, capabilities, DEFAULT_FILE_NAME)
    }

    fn new_impl(content: &str, capabilities: Option<String>, default_name: &str) -> Session {
        let data = parse_test_data(content, default_name);
        let inner_fs = Arc::new(InMemoryFS::new());
        let mut contents = std::collections::BTreeMap::new();
        let mut file_names = Vec::new();
        for f in &data.files {
            let path = project_path(&f.file_name);
            inner_fs.insert_file(&path, &f.content);
            contents.insert(f.file_name.clone(), f.content.clone());
            file_names.push(path);
        }
        let active_file = data
            .files
            .first()
            .map(|f| f.file_name.clone())
            .unwrap_or_else(|| DEFAULT_FILE_NAME.to_string());
        let prefs = Arc::new(Mutex::new(crate::ls::lsutil::new_default_user_preferences()));
        let mut s = Session {
            data,
            active_file,
            cursor: None,
            enable_formatting: true,
            contents,
            capabilities,
            inner_fs,
            prefs: prefs.clone(),
            service: None,
        };
        s.rebuild_service(file_names);
        s
    }

    pub(crate) fn rebuild_service(&mut self, file_names: Vec<String>) {
        let dyn_fs: Arc<dyn FS> = Arc::clone(&self.inner_fs) as _;
        let fs = Arc::new(BundledFS::new(dyn_fs));
        // 合并全局与各文件 @options（对齐 Go fourslash：文件头选项作用于整个测试工程）
        let mut merged_options = self.data.global_options.clone();
        for f in &self.data.files {
            for (k, v) in &f.file_options {
                merged_options.insert(k.clone(), v.clone());
            }
        }
        // Go fourslash 基底默认（fourslash.go:205）：skipDefaultLibCheck /
        // target=latest / jsx=preserve，可被 @options 覆盖
        merged_options
            .entry("skipDefaultLibCheck".to_string())
            .or_insert_with(|| String::new());
        merged_options
            .entry("target".to_string())
            .or_insert_with(|| "latest".to_string());
        merged_options
            .entry("jsx".to_string())
            .or_insert_with(|| "preserve".to_string());
        let mut args: Vec<String> = Vec::new();
        for (k, v) in &merged_options {
            if v.is_empty() {
                args.push(format!("--{k}"));
            } else {
                args.push(format!("--{k}={v}"));
            }
        }
        args.extend(file_names);
        let parsed = parse_command_line(&args, PROJECT_ROOT, Some(fs.as_ref()));
        let host: Arc<dyn CompilerHost> = Arc::new(CompilerHostImpl::new(
            fs.clone(),
            PROJECT_ROOT.to_string(),
            lib_path(),
        ));
        let program = Arc::new(Program::new(ProgramOptions {
            config: parsed,
            host,
        }));
        let active = project_path(&self.active_file);
        let ls_host = Box::new(FourslashHost {
            fs,
            prefs: Arc::clone(&self.prefs),
        });
        self.service = Some(LanguageService::new(
            Path::from(PROJECT_ROOT),
            program,
            ls_host,
            &active,
        ));
    }

    pub fn file_content(&self, name: &str) -> &str {
        self.contents.get(name).unwrap_or_else(|| {
            // 用例代码可能用 @Filename 原样名（无 / 前缀）查文件：按解析侧
            // 的规范化（GetNormalizedAbsolutePath(x, "/")）兜底
            let normalized = if name.starts_with('/') {
                name.to_string()
            } else {
                format!("/{name}")
            };
            self.contents
                .get(&normalized)
                .unwrap_or_else(|| panic!("文件不存在: {name}"))
        })
    }

    pub fn set_file_content(&mut self, name: &str, content: String) {
        self.inner_fs
            .insert_file(&project_path(name), &content);
        self.contents.insert(name.to_string(), content);
        let file_names: Vec<String> = self
            .data
            .files
            .iter()
            .map(|f| project_path(&f.file_name))
            .collect();
        self.rebuild_service(file_names);
    }

    pub fn active_path(&self) -> String {
        project_path(&self.active_file)
    }

    pub fn marker(&self, name: &str) -> &Marker {
        self.data
            .markers
            .iter()
            .find(|m| m.name.as_deref() == Some(name))
            .unwrap_or_else(|| panic!("标记不存在: {name}"))
    }

    pub fn ranges_in(&self, file: &str) -> Vec<&RangeMarker> {
        self.data
            .ranges
            .iter()
            .filter(|r| r.file_name == file)
            .collect()
    }
}
