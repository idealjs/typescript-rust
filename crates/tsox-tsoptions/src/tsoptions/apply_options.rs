#![allow(unused_imports)]

use super::*;

pub(crate) fn apply_options(options: &HashMap<String, OptValue>, out: &mut CompilerOptions) {
    for (name, value) in options {
        match name.as_str() {
            "target" => {
                if let Some(s) = value.as_str() {
                    out.target = parse_script_target(s);
                }
            }
            "module" => {
                if let Some(s) = value.as_str() {
                    out.module = parse_module_kind(s);
                }
            }
            "moduleResolution" => {
                if let Some(s) = value.as_str() {
                    out.module_resolution = parse_module_resolution(s);
                }
            }
            "jsx" => {
                if let Some(s) = value.as_str() {
                    out.jsx = parse_jsx_emit(s);
                }
            }
            "newLine" => {
                if let Some(s) = value.as_str() {
                    out.new_line = match s.to_lowercase().as_str() {
                        "crlf" => NewLineKind::CRLF,
                        "lf" => NewLineKind::LF,
                        _ => NewLineKind::None,
                    };
                }
            }
            "moduleDetection" => {
                if let Some(s) = value.as_str() {
                    out.module_detection = match s.to_lowercase().as_str() {
                        "auto" => ModuleDetectionKind::Auto,
                        "legacy" => ModuleDetectionKind::Legacy,
                        "force" => ModuleDetectionKind::Force,
                        _ => ModuleDetectionKind::None,
                    };
                }
            }
            "lib" => {
                if let Some(list) = value.as_list() {
                    out.lib = list.to_vec();
                }
            }
            "types" => {
                if let Some(list) = value.as_list() {
                    out.types = list.to_vec();
                }
            }
            "typeRoots" => {
                if let Some(list) = value.as_list() {
                    out.type_roots = list.to_vec();
                }
            }
            "rootDirs" => {
                if let Some(list) = value.as_list() {
                    out.root_dirs = list.to_vec();
                }
            }
            "outDir" => {
                if let Some(s) = value.as_str() {
                    out.out_dir = s.to_string();
                }
            }
            "outFile" => {
                if let Some(s) = value.as_str() {
                    out.out_file = s.to_string();
                }
            }
            "rootDir" => {
                if let Some(s) = value.as_str() {
                    out.root_dir = s.to_string();
                }
            }
            "baseUrl" => {
                if let Some(s) = value.as_str() {
                    out.base_url = s.to_string();
                }
            }
            "project" => {
                if let Some(s) = value.as_str() {
                    out.project = s.to_string();
                }
            }
            "declarationDir" => {
                if let Some(s) = value.as_str() {
                    out.declaration_dir = s.to_string();
                }
            }
            "tsBuildInfoFile" => {
                if let Some(s) = value.as_str() {
                    out.ts_build_info_file = s.to_string();
                }
            }
            "sourceRoot" => {
                if let Some(s) = value.as_str() {
                    out.source_root = s.to_string();
                }
            }
            "mapRoot" => {
                if let Some(s) = value.as_str() {
                    out.map_root = s.to_string();
                }
            }
            "jsxFactory" => {
                if let Some(s) = value.as_str() {
                    out.jsx_factory = s.to_string();
                }
            }
            "jsxFragmentFactory" => {
                if let Some(s) = value.as_str() {
                    out.jsx_fragment_factory = s.to_string();
                }
            }
            "jsxImportSource" => {
                if let Some(s) = value.as_str() {
                    out.jsx_import_source = s.to_string();
                }
            }
            "reactNamespace" => {
                if let Some(s) = value.as_str() {
                    out.react_namespace = s.to_string();
                }
            }
            "locale" => {
                if let Some(s) = value.as_str() {
                    out.locale = s.to_string();
                }
            }
            "generateTrace" => {
                if let Some(s) = value.as_str() {
                    out.generate_trace = s.to_string();
                }
            }
            _ => {
                if let Some(b) = value.as_bool() {
                    set_bool(out, name, b);
                }
            }
        }
    }
}

pub(crate) fn apply_build_options(options: &HashMap<String, OptValue>, out: &mut BuildOptions) {
    for (name, value) in options {
        match name.as_str() {
            "clean" => {
                if let Some(b) = value.as_bool() {
                    out.clean = Tristate::from(b);
                }
            }
            "dry" => {
                if let Some(b) = value.as_bool() {
                    out.dry = Tristate::from(b);
                }
            }
            "force" => {
                if let Some(b) = value.as_bool() {
                    out.force = Tristate::from(b);
                }
            }
            "verbose" => {
                if let Some(b) = value.as_bool() {
                    out.verbose = Tristate::from(b);
                }
            }
            "stopBuildOnErrors" => {
                if let Some(b) = value.as_bool() {
                    out.stop_build_on_errors = Tristate::from(b);
                }
            }
            "builders" => {
                if let OptValue::Num(n) = value {
                    out.builders = Some(*n as i32);
                }
            }
            _ => {}
        }
    }
}

pub(crate) fn apply_watch_options(options: &HashMap<String, OptValue>, out: &mut WatchOptions) {
    for (name, value) in options {
        match name.as_str() {
            "watchInterval" => {
                if let OptValue::Num(n) = value {
                    out.interval = Some(*n as i32);
                } else if matches!(value, OptValue::Null) {
                    out.interval = None;
                }
            }
            "watchFile" => {
                if let Some(s) = value.as_str() {
                    out.file_kind = parse_watch_file_kind(s).unwrap_or(WatchFileKind::None);
                } else if matches!(value, OptValue::Null) {
                    out.file_kind = WatchFileKind::None;
                }
            }
            "watchDirectory" => {
                if let Some(s) = value.as_str() {
                    out.directory_kind =
                        parse_watch_directory_kind(s).unwrap_or(WatchDirectoryKind::None);
                } else if matches!(value, OptValue::Null) {
                    out.directory_kind = WatchDirectoryKind::None;
                }
            }
            "fallbackPolling" => {
                if let Some(s) = value.as_str() {
                    out.fallback_polling = parse_polling_kind(s).unwrap_or(PollingKind::None);
                } else if matches!(value, OptValue::Null) {
                    out.fallback_polling = PollingKind::None;
                }
            }
            "synchronousWatchDirectory" => {
                if let Some(b) = value.as_bool() {
                    out.sync_watch_dir = Tristate::from(b);
                } else if matches!(value, OptValue::Null) {
                    out.sync_watch_dir = Tristate::Unknown;
                }
            }
            "excludeDirectories" => {
                if let Some(list) = value.as_list() {
                    out.exclude_dir = list.to_vec();
                }
            }
            "excludeFiles" => {
                if let Some(list) = value.as_list() {
                    out.exclude_files = list.to_vec();
                }
            }
            _ => {}
        }
    }
}

pub(crate) fn parse_script_target(s: &str) -> ScriptTarget {
    let s = s.to_lowercase();
    let s = s.replace('-', "");
    match s.as_str() {
        "es3" => ScriptTarget::ES5,
        "es5" => ScriptTarget::ES5,
        "es6" | "es2015" => ScriptTarget::ES2015,
        "es2016" => ScriptTarget::ES2016,
        "es2017" => ScriptTarget::ES2017,
        "es2018" => ScriptTarget::ES2018,
        "es2019" => ScriptTarget::ES2019,
        "es2020" => ScriptTarget::ES2020,
        "es2021" => ScriptTarget::ES2021,
        "es2022" => ScriptTarget::ES2022,
        "es2023" => ScriptTarget::ES2023,
        "es2024" => ScriptTarget::ES2024,
        "es2025" => ScriptTarget::ES2025,
        "esnext" => ScriptTarget::ESNext,
        "json" => ScriptTarget::JSON,
        _ => ScriptTarget::None,
    }
}

pub(crate) fn parse_module_kind(s: &str) -> ModuleKind {
    match s.to_lowercase().as_str() {
        "commonjs" => ModuleKind::CommonJS,
        "amd" => ModuleKind::AMD,
        "umd" => ModuleKind::UMD,
        "system" => ModuleKind::System,
        "es6" | "es2015" => ModuleKind::ES2015,
        "es2020" => ModuleKind::ES2020,
        "es2022" => ModuleKind::ES2022,
        "esnext" => ModuleKind::ESNext,
        "node16" => ModuleKind::Node16,
        "node18" => ModuleKind::Node18,
        "node20" => ModuleKind::Node20,
        "nodenext" => ModuleKind::NodeNext,
        "preserve" => ModuleKind::Preserve,
        _ => ModuleKind::None,
    }
}
