#![allow(unused_imports)]

use super::*;

impl ExtendedConfigCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub(crate) fn get_or_parse(
        &mut self,
        resolved_path: &str,
        config_file_name: &str,
        current_dir: &str,
        fs: &dyn FS,
        resolution_stack: &[String],
    ) -> ParsedCommandLine {
        if resolution_stack.iter().any(|p| p == resolved_path) {
            return get_parsed_command_line_of_config_file_with_stack(
                config_file_name,
                &CompilerOptions::default(),
                current_dir,
                fs,
                resolution_stack,
                self,
            );
        }
        if let Some(cached) = self.entries.get(resolved_path) {
            return cached.clone();
        }
        let parsed = get_parsed_command_line_of_config_file_with_stack(
            config_file_name,
            &CompilerOptions::default(),
            current_dir,
            fs,
            resolution_stack,
            self,
        );
        self.entries
            .insert(resolved_path.to_string(), parsed.clone());
        parsed
    }
}

#[derive(Debug, Clone, Default)]
pub struct BuildOptions {
    pub clean: Tristate,
    pub dry: Tristate,
    pub force: Tristate,
    pub verbose: Tristate,
    pub stop_build_on_errors: Tristate,
    pub builders: Option<i32>,
}

#[derive(Debug, Clone, Default)]
pub struct ParsedBuildCommandLine {
    pub build_options: BuildOptions,
    pub compiler_options: CompilerOptions,
    pub projects: Vec<String>,
    pub errors: Vec<Diagnostic>,

    pub watch_options: WatchOptions,
    pub(crate) current_dir: String,
}

impl ParsedBuildCommandLine {
    pub fn resolved_project_paths(&self) -> Vec<String> {
        self.projects
            .iter()
            .map(|project| tspath::get_normalized_absolute_path(project, &self.current_dir))
            .collect()
    }
}

impl Diagnostic {
    pub fn with_text(self, text: impl Into<String>) -> Diagnostic {
        Diagnostic {
            file: self.file,
            loc: self.loc,
            code: self.code,
            category: self.category,
            message: None,
            message_key: self.message_key,
            message_args: vec![text.into()],
            message_chain: self.message_chain,
            related_information: self.related_information,
            reports_unnecessary: self.reports_unnecessary,
            reports_deprecated: self.reports_deprecated,
            skipped_on_no_emit: self.skipped_on_no_emit,
        }
    }
}

pub(crate) fn err(text: impl Into<String>) -> Diagnostic {
    Diagnostic::new(None, TextRange::undefined(), new_ad_hoc_message(""), vec![]).with_text(text)
}

pub fn parse_command_line(
    args: &[String],
    current_dir: &str,
    fs: Option<&dyn FS>,
) -> ParsedCommandLine {
    let (options, watch_options_map, file_names, errors) =
        parse_command_line_worker(args, current_dir, fs, find_option, ParseMode::Compiler);

    let mut compiler_options = CompilerOptions::default();
    apply_options(&options, &mut compiler_options);
    let watch = compiler_options.watch.is_true();
    let mut watch_options = WatchOptions::default();
    apply_watch_options(&watch_options_map, &mut watch_options);

    let file_names = file_names
        .iter()
        .map(|f| tspath::get_normalized_absolute_path(f, current_dir))
        .collect();

    ParsedCommandLine {
        compiler_options,
        file_names,
        errors,
        config_file_name: String::new(),
        raw_options: None,
        include: Vec::new(),
        exclude: Vec::new(),
        files_spec: Vec::new(),
        has_include_spec: false,
        has_exclude_spec: false,
        has_files_spec: false,
        references: Vec::new(),
        compile_on_save: None,
        watch,
        watch_options,
    }
}

pub fn parse_build_command_line(
    args: &[String],
    current_dir: &str,
    fs: Option<&dyn FS>,
) -> ParsedBuildCommandLine {
    let (options, watch_options_map, mut projects, mut errors) =
        parse_command_line_worker(args, current_dir, fs, find_build_option, ParseMode::Build);

    if projects.is_empty() {
        projects.push(".".to_string());
    }

    let mut compiler_options = CompilerOptions::default();
    apply_options(&options, &mut compiler_options);

    let mut build_options = BuildOptions::default();
    apply_build_options(&options, &mut build_options);

    let mut watch_options = WatchOptions::default();
    apply_watch_options(&watch_options_map, &mut watch_options);

    if build_options.clean.is_true() && build_options.force.is_true() {
        errors.push(Diagnostic::new(
            None,
            TextRange::undefined(),
            OPTIONS_0_AND_1_CANNOT_BE_COMBINED,
            vec!["clean".to_string(), "force".to_string()],
        ));
    }
    if build_options.clean.is_true() && build_options.verbose.is_true() {
        errors.push(Diagnostic::new(
            None,
            TextRange::undefined(),
            OPTIONS_0_AND_1_CANNOT_BE_COMBINED,
            vec!["clean".to_string(), "verbose".to_string()],
        ));
    }
    if build_options.clean.is_true() && compiler_options.watch.is_true() {
        errors.push(Diagnostic::new(
            None,
            TextRange::undefined(),
            OPTIONS_0_AND_1_CANNOT_BE_COMBINED,
            vec!["clean".to_string(), "watch".to_string()],
        ));
    }
    if compiler_options.watch.is_true() && build_options.dry.is_true() {
        errors.push(Diagnostic::new(
            None,
            TextRange::undefined(),
            OPTIONS_0_AND_1_CANNOT_BE_COMBINED,
            vec!["watch".to_string(), "dry".to_string()],
        ));
    }

    ParsedBuildCommandLine {
        build_options,
        compiler_options,
        projects,
        errors,
        watch_options,
        current_dir: current_dir.to_string(),
    }
}
