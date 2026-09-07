#![allow(unused_imports)]

use super::*;

pub(crate) fn parse_module_resolution(s: &str) -> ModuleResolutionKind {
    match s.to_lowercase().as_str() {
        "classic" => ModuleResolutionKind::Classic,
        "node" | "node10" => ModuleResolutionKind::Node10,
        "node16" => ModuleResolutionKind::Node16,
        "nodenext" => ModuleResolutionKind::NodeNext,
        "bundler" => ModuleResolutionKind::Bundler,
        _ => ModuleResolutionKind::Unknown,
    }
}

pub(crate) fn parse_jsx_emit(s: &str) -> JsxEmit {
    match s.to_lowercase().as_str() {
        "preserve" => JsxEmit::Preserve,
        "react" => JsxEmit::React,
        "react-native" => JsxEmit::ReactNative,
        "react-jsx" => JsxEmit::ReactJSX,
        "react-jsxdev" => JsxEmit::ReactJSXDev,
        _ => JsxEmit::None,
    }
}

pub fn script_target_name(t: ScriptTarget) -> Option<&'static str> {
    match t {
        ScriptTarget::ES5 => Some("es5"),
        ScriptTarget::ES2015 => Some("es2015"),
        ScriptTarget::ES2016 => Some("es2016"),
        ScriptTarget::ES2017 => Some("es2017"),
        ScriptTarget::ES2018 => Some("es2018"),
        ScriptTarget::ES2019 => Some("es2019"),
        ScriptTarget::ES2020 => Some("es2020"),
        ScriptTarget::ES2021 => Some("es2021"),
        ScriptTarget::ES2022 => Some("es2022"),
        ScriptTarget::ES2023 => Some("es2023"),
        ScriptTarget::ES2024 => Some("es2024"),
        ScriptTarget::ES2025 => Some("es2025"),
        ScriptTarget::ESNext => Some("esnext"),
        ScriptTarget::JSON => Some("json"),
        ScriptTarget::None => None,
    }
}

pub fn module_kind_name(m: ModuleKind) -> Option<&'static str> {
    match m {
        ModuleKind::CommonJS => Some("commonjs"),
        ModuleKind::AMD => Some("amd"),
        ModuleKind::UMD => Some("umd"),
        ModuleKind::System => Some("system"),
        ModuleKind::ES2015 => Some("es2015"),
        ModuleKind::ES2020 => Some("es2020"),
        ModuleKind::ES2022 => Some("es2022"),
        ModuleKind::ESNext => Some("esnext"),
        ModuleKind::Node16 => Some("node16"),
        ModuleKind::Node18 => Some("node18"),
        ModuleKind::Node20 => Some("node20"),
        ModuleKind::NodeNext => Some("nodenext"),
        ModuleKind::Preserve => Some("preserve"),
        ModuleKind::None => None,
    }
}

pub fn module_resolution_name(r: ModuleResolutionKind) -> Option<&'static str> {
    match r {
        ModuleResolutionKind::Classic => Some("classic"),
        ModuleResolutionKind::Node10 => Some("node10"),
        ModuleResolutionKind::Node16 => Some("node16"),
        ModuleResolutionKind::NodeNext => Some("nodenext"),
        ModuleResolutionKind::Bundler => Some("bundler"),
        ModuleResolutionKind::Unknown => None,
    }
}

pub fn jsx_emit_name(j: JsxEmit) -> Option<&'static str> {
    match j {
        JsxEmit::Preserve => Some("preserve"),
        JsxEmit::React => Some("react"),
        JsxEmit::ReactNative => Some("react-native"),
        JsxEmit::ReactJSX => Some("react-jsx"),
        JsxEmit::ReactJSXDev => Some("react-jsxdev"),
        JsxEmit::None => None,
    }
}

pub fn module_detection_name(d: ModuleDetectionKind) -> Option<&'static str> {
    match d {
        ModuleDetectionKind::Auto => Some("auto"),
        ModuleDetectionKind::Force => Some("force"),
        ModuleDetectionKind::Legacy => Some("legacy"),
        ModuleDetectionKind::None => None,
    }
}

pub fn new_line_name(n: NewLineKind) -> Option<&'static str> {
    match n {
        NewLineKind::CRLF => Some("crlf"),
        NewLineKind::LF => Some("lf"),
        NewLineKind::None => None,
    }
}

pub fn get_parsed_command_line_of_config_file(
    config_file_name: &str,
    base_options: &CompilerOptions,
    current_dir: &str,
    fs: &dyn FS,
) -> ParsedCommandLine {
    let mut cache = ExtendedConfigCache::new();
    get_parsed_command_line_of_config_file_with_stack(
        config_file_name,
        base_options,
        current_dir,
        fs,
        &[],
        &mut cache,
    )
}
