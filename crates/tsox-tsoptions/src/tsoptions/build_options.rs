#![allow(unused_imports)]

use super::*;

pub const BUILD_OPTIONS: &[OptionDecl] = &[
    OptionDecl {
        name: "build",
        short_name: Some("b"),
        kind: OptionKind::Boolean,
        is_file_path: false,
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "verbose",
        short_name: Some("v"),
        kind: OptionKind::Boolean,
        is_file_path: false,
        description: "Enable verbose logging in build mode.",
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "dry",
        short_name: Some("d"),
        kind: OptionKind::Boolean,
        is_file_path: false,
        description: "Show what would be built (dry run) in build mode.",
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "force",
        short_name: Some("f"),
        kind: OptionKind::Boolean,
        is_file_path: false,
        description: "Build all projects, including those that are up to date.",
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "clean",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        description: "Delete the outputs of all projects in build mode.",
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "builders",
        short_name: None,
        kind: OptionKind::Number,
        is_file_path: false,
        min_value: Some(1),
        extra_validation: ExtraValidation::MinValue,
        description: "Number of concurrent build workers in build mode.",
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "stopBuildOnErrors",
        short_name: None,
        kind: OptionKind::Boolean,
        is_file_path: false,
        description: "Stop building projects immediately after an error is reported in build mode.",
        ..DEFAULT_DECL
    },
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParseMode {
    Compiler,
    Build,
}

pub(crate) static WATCH_FILE_ENUM_VALUES: &[&str] = &[
    "fixedpollinginterval",
    "prioritypollinginterval",
    "dynamicprioritypolling",
    "fixedchunksizepolling",
    "usefsevents",
    "usefseventsonparentdirectory",
];
pub(crate) static WATCH_DIRECTORY_ENUM_VALUES: &[&str] = &[
    "usefsevents",
    "fixedpollinginterval",
    "dynamicprioritypolling",
    "fixedchunksizepolling",
];
pub(crate) static FALLBACK_POLLING_ENUM_VALUES: &[&str] = &[
    "fixedinterval",
    "priorityinterval",
    "dynamicpriority",
    "fixedchunksize",
];

pub const OPTIONS_FOR_WATCH: &[OptionDecl] = &[
    OptionDecl {
        name: "watchInterval",
        short_name: None,
        kind: OptionKind::Number,
        description: "Specify the polling interval for watch mode (milliseconds).",
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "watchFile",
        short_name: None,
        kind: OptionKind::Enum,
        enum_values: Some(WATCH_FILE_ENUM_VALUES),
        description: "Specify how the TypeScript watch mode works.",
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "watchDirectory",
        short_name: None,
        kind: OptionKind::Enum,
        enum_values: Some(WATCH_DIRECTORY_ENUM_VALUES),
        description: "Specify how directories are watched on systems that lack recursive file watching functionality.",
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "fallbackPolling",
        short_name: None,
        kind: OptionKind::Enum,
        enum_values: Some(FALLBACK_POLLING_ENUM_VALUES),
        description: "Specify what approach the watcher should use if the system runs out of native file watchers.",
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "synchronousWatchDirectory",
        short_name: None,
        kind: OptionKind::Boolean,
        description: "Synchronously call callbacks and update the state of directory watchers on platforms that don't support recursive watching natively.",
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "excludeDirectories",
        short_name: None,
        kind: OptionKind::List,
        is_file_path: true,
        description: "Remove a list of directories from the watch process.",
        ..DEFAULT_DECL
    },
    OptionDecl {
        name: "excludeFiles",
        short_name: None,
        kind: OptionKind::List,
        is_file_path: true,
        description: "Remove a list of files from the watch mode's processing.",
        ..DEFAULT_DECL
    },
];

pub(crate) fn decl_matches(o: &OptionDecl, name: &str) -> bool { ::tsox_core::fntrace::enter("decl_matches"); 
    o.name.eq_ignore_ascii_case(name)
        || o.short_name
            .map(|s| s.eq_ignore_ascii_case(name))
            .unwrap_or(false)
}

pub(crate) fn find_option(name: &str) -> Option<&'static OptionDecl> { ::tsox_core::fntrace::enter("find_option"); 
    OPTIONS.iter().find(|o| decl_matches(o, name))
}

pub(crate) fn find_build_only_option(name: &str) -> Option<&'static OptionDecl> { ::tsox_core::fntrace::enter("find_build_only_option"); 
    BUILD_OPTIONS.iter().find(|o| decl_matches(o, name))
}

pub(crate) fn find_build_option(name: &str) -> Option<&'static OptionDecl> { ::tsox_core::fntrace::enter("find_build_option"); 
    BUILD_OPTIONS
        .iter()
        .chain(OPTIONS.iter())
        .find(|o| decl_matches(o, name))
}

pub(crate) fn did_you_mean_build_option(input: &str) -> Option<String> { ::tsox_core::fntrace::enter("did_you_mean_build_option"); 
    let input_lower = input.to_lowercase();
    let mut best: Option<(usize, &str)> = None;
    for opt in BUILD_OPTIONS.iter().chain(OPTIONS.iter()) {
        let name = opt.name.to_lowercase();
        let dist = levenshtein(&input_lower, &name);

        if dist <= 3 && best.map_or(true, |(d, _)| dist < d) {
            best = Some((dist, opt.name));
        }
    }
    best.map(|(_, name)| name.to_string())
}

pub(crate) fn levenshtein(a: &str, b: &str) -> usize { ::tsox_core::fntrace::enter("levenshtein"); 
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let m = a.len();
    let n = b.len();
    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr = vec![0usize; n + 1];
    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
}

pub(crate) fn find_watch_option(name: &str) -> Option<&'static OptionDecl> { ::tsox_core::fntrace::enter("find_watch_option"); 
    OPTIONS_FOR_WATCH.iter().find(|o| decl_matches(o, name))
}

#[derive(Debug, Clone)]
pub enum OptValue {
    Bool(bool),
    Str(String),
    Num(i64),
    List(Vec<String>),
    Null,
}

impl OptValue {
    pub(crate) fn as_bool(&self) -> Option<bool> { ::tsox_core::fntrace::enter("as_bool"); 
        match self {
            OptValue::Bool(b) => Some(*b),
            _ => None,
        }
    }
    pub(crate) fn as_str(&self) -> Option<&str> { ::tsox_core::fntrace::enter("as_str"); 
        match self {
            OptValue::Str(s) => Some(s),
            _ => None,
        }
    }
    pub(crate) fn as_list(&self) -> Option<&[String]> { ::tsox_core::fntrace::enter("as_list"); 
        match self {
            OptValue::List(v) => Some(v),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ParsedCommandLine {
    pub compiler_options: CompilerOptions,
    pub file_names: Vec<String>,
    pub errors: Vec<Diagnostic>,
    pub config_file_name: String,

    pub raw_options: Option<tsox_core::json::Value>,

    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub files_spec: Vec<String>,
    pub has_include_spec: bool,
    pub has_exclude_spec: bool,
    pub has_files_spec: bool,
    pub references: Vec<tsox_core::core::project_reference::ProjectReference>,
    pub compile_on_save: Option<bool>,
    pub watch: bool,

    pub watch_options: WatchOptions,

    pub config_file: Option<std::sync::Arc<crate::mig::m5j::TsConfigSourceFile>>,
    pub type_acquisition: Option<tsox_core::core::mig::m3k::TypeAcquisition>,
    pub content_mappers: Vec<crate::mig::m5h_3::ContentMapper>,
    pub literal_file_names_len: usize,
    pub compare_paths_options: tsox_core::tspath::ComparePathsOptions,

    pub source_to_project_reference: tsox_core::collections::ordered_map::OrderedMap<
        String,
        crate::mig::m5h_3::SourceOutputAndProjectReference,
    >,
    pub output_dts_to_project_reference: tsox_core::collections::ordered_map::OrderedMap<
        String,
        crate::mig::m5h_3::SourceOutputAndProjectReference,
    >,
    pub source_and_output_maps_once: std::sync::OnceLock<()>,

    pub common_source_directory: String,
    pub common_source_directory_once: std::sync::OnceLock<()>,

    pub resolved_project_reference_paths: Vec<String>,
    pub resolved_project_reference_paths_once: std::sync::OnceLock<()>,

    pub file_names_by_path: tsox_core::collections::ordered_map::OrderedMap<String, String>,
    pub file_names_by_path_once: std::sync::OnceLock<()>,

    pub locale: tsox_core::locale::Locale,
    pub locale_once: std::sync::OnceLock<()>,
}

#[derive(Default)]
pub struct ExtendedConfigCache {
    pub(crate) entries: HashMap<String, ParsedCommandLine>,
}
