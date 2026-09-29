#![allow(unused_imports, dead_code, unused_variables)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex, Once};
use std::time::SystemTime;

use tsox_core::collections::set::Set;
use tsox_core::collections::syncmap::SyncMap;
use tsox_core::tspath::{self, Path};
use tsox_tsoptions::tsoptions::{parse_build_command_line, parse_command_line, ParsedCommandLine};
use tsox_tsoptions::vfs::FS;

use super::m5a::CommandLineTesting;
use crate::execute::CommandLineResult;
use crate::execute::ExitStatus;
use crate::execute::System;

use tsox_compile::compiler::CompilerHost;

pub trait Host {
    fn fs(&self) -> Arc<dyn FS>;
    fn get_m_time(&self, file_name: &str) -> SystemTime;
    fn set_m_time(&self, file_name: &str, m_time: SystemTime) -> std::io::Result<()>;
}

pub struct HostImpl {
    pub host: Arc<dyn CompilerHost>,
}

impl Host for HostImpl {
    fn fs(&self) -> Arc<dyn FS> {
        self.host.fs_arc()
    }

    fn get_m_time(&self, file_name: &str) -> SystemTime {
        get_m_time(self.host.as_ref(), file_name)
    }

    fn set_m_time(&self, file_name: &str, m_time: SystemTime) -> std::io::Result<()> {
        self.host.fs().chtimes(file_name, m_time, m_time)
    }
}

pub fn create_host(compiler_host: Arc<dyn CompilerHost>) -> HostImpl {
    HostImpl { host: compiler_host }
}

pub fn get_m_time(host: &dyn CompilerHost, file_name: &str) -> SystemTime {
    match host.fs().stat(file_name) {
        Some(stat) => stat.modified,
        None => SystemTime::UNIX_EPOCH,
    }
}

pub trait BuildInfoReader {
    fn read_build_info(&self, config: &ParsedCommandLine) -> Option<crate::mig::m4y_2::BuildInfo>;
}

pub struct BuildInfoReaderImpl {
    pub host: Arc<dyn CompilerHost>,
}

impl BuildInfoReader for BuildInfoReaderImpl {
    fn read_build_info(&self, config: &ParsedCommandLine) -> Option<crate::mig::m4y_2::BuildInfo> {
        let build_info_file_name = config.get_build_info_file_name();
        if build_info_file_name.is_empty() {
            return None;
        }

        let data = self.host.fs().read_file(&build_info_file_name)?;
        let build_info: crate::mig::m4y_2::BuildInfo = serde_json::from_str(&data).ok()?;
        Some(build_info)
    }
}

pub fn new_build_info_reader(host: Arc<dyn CompilerHost>) -> BuildInfoReaderImpl {
    BuildInfoReaderImpl { host }
}

pub fn read_build_info_program(
    config: &ParsedCommandLine,
    reader: &dyn BuildInfoReader,
    host: &dyn CompilerHost,
) -> Option<crate::mig::m4z2::Program> {
    let build_info = reader.read_build_info(config)?;
    if !build_info.is_valid_version() || !build_info.is_incremental() {
        return None;
    }

    let content_mapper_identities =
        crate::mig::m4y_2::content_mapper_identities(host.content_mapper_project().as_deref()).ok()?;
    if !build_info.content_mapper_identities_match(&content_mapper_identities) {
        return None;
    }

    let incremental_program = crate::mig::m4z2::Program {
        snapshot: crate::mig::m4y_3::build_info_to_snapshot(&build_info, config, host),
        program: None,
        host: None,
        testing_data: None,
        nested_emit_now: None,
        nested_emit_depth: Mutex::new(0),
        nested_emit_start: Mutex::new(SystemTime::UNIX_EPOCH),
        nested_emit_time: Mutex::new(std::time::Duration::ZERO),
    };
    Some(incremental_program)
}

pub struct ReferenceMap {
    pub references: SyncMap<Path, Arc<Set<Path>>>,
    pub referenced_by: std::sync::Mutex<Option<HashMap<Path, Set<Path>>>>,
    pub reference_by: Once,
}

impl ReferenceMap {
    pub fn store_references(&self, path: Path, refs: Arc<Set<Path>>) {
        self.references.store(path, refs);
    }

    pub fn get_references(&self, path: &Path) -> Option<Arc<Set<Path>>> {
        self.references.load(path)
    }

    pub fn get_paths_with_references(&self) -> Vec<Path> {
        self.references.keys()
    }

    pub fn get_referenced_by(&self, path: &Path) -> Vec<Path> {
        self.reference_by.call_once(|| {
            let mut referenced_by: HashMap<Path, Set<Path>> = HashMap::new();
            self.references.for_each(|key, value| {
                for r#ref in value.iter() {
                    let set = referenced_by.entry(r#ref.clone()).or_default();
                    set.add(key.clone());
                }
                true
            });
            *self.referenced_by.lock().unwrap() = Some(referenced_by);
        });
        let referenced_by = self.referenced_by.lock().unwrap();
        match referenced_by.as_ref().and_then(|m| m.get(path)) {
            Some(refs) => refs.iter().cloned().collect(),
            None => Vec::new(),
        }
    }
}

pub fn command_line(
    sys: &dyn System,
    command_line_args: &[String],
    testing: Option<&dyn CommandLineTesting>,
) -> CommandLineResult {
    if !command_line_args.is_empty() {
        match command_line_args[0].to_lowercase().as_str() {
            "-b" | "--b" | "-build" | "--build" => {
                return crate::execute::version::tsc_build_compilation(
                    sys,
                    parse_build_command_line(
                        command_line_args,
                        sys.current_directory(),
                        Some(sys.fs().as_ref()),
                    ),
                );
            }
            _ => {}
        }
    }

    crate::execute::tsc_compilation::tsc_compilation(
        sys,
        parse_command_line(
            command_line_args,
            sys.current_directory(),
            Some(sys.fs().as_ref()),
        ),
    )
}

pub fn fmt_main(sys: &dyn System, input: &str, output: &str) -> ExitStatus {
    let input = tspath::to_path(
        input,
        sys.current_directory(),
        sys.fs().use_case_sensitive_file_names(),
    )
    .to_string();
    let output = tspath::to_path(
        output,
        sys.current_directory(),
        sys.fs().use_case_sensitive_file_names(),
    )
    .to_string();
    let file_content = match sys.fs().read_file(&input) {
        Some(content) => content,
        None => {
            writeln!(sys.writer(), "File not found: {input}").ok();
            return ExitStatus::NotImplemented;
        }
    };
    let text = file_content;
    let source_file = tsox_frontend::parser::Parser::parse_source_file_text(&input, text.clone());
    let ctx = tsox_frontend::format::with_format_code_settings(
        tsox_frontend::format::get_default_format_code_settings(),
        "\n",
    );
    let edits = tsox_frontend::format::format_document(&ctx, &Arc::new(source_file));
    let edits: Vec<tsox_core::core::text_change::TextChange> = edits
        .into_iter()
        .map(|c| tsox_core::core::text_change::TextChange {
            range: tsox_core::core::text::TextRange::new(c.pos, c.end),
            new_text: c.new_text,
        })
        .collect();
    let new_text = tsox_core::core::text_change::apply_bulk_edits(&text, &edits);

    if sys.fs().write_file(&output, &new_text).is_err() {
        return ExitStatus::NotImplemented;
    }
    ExitStatus::Success
}

pub fn find_config_file(search_path: &str, file_exists: &dyn Fn(&str) -> bool, config_name: &str) -> String {
    let mut result = String::new();
    tspath::for_each_ancestor_directory(search_path, |ancestor| {
        let full_config_name = tspath::combine_paths(ancestor, &[config_name]);
        if file_exists(&full_config_name) {
            result = full_config_name;
            true
        } else {
            false
        }
    });
    result
}
