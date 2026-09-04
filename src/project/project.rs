#![allow(dead_code)]

use std::collections::HashSet;
use std::sync::{Arc, Mutex, OnceLock};

use crate::compiler::Program;
use crate::tsoptions::ParsedCommandLine;
use crate::tspath::Path;

pub const INFERRED_PROJECT_NAME: &str = "/dev/null/inferred";

pub const HR: &str = "-----------------------------------------------";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Inferred,
    Configured,
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kind::Inferred => write!(f, "Inferred"),
            Kind::Configured => write!(f, "Configured"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProgramUpdateKind {
    #[default]
    None,
    Cloned,
    SameFileNames,
    NewFiles,
}

#[derive(Default)]
pub struct CreateProgramResult {
    pub program: Option<Arc<Program>>,
    pub update_kind: ProgramUpdateKind,
}

pub struct Project {
    pub kind: Kind,
    pub current_directory: String,
    pub config_file_name: String,
    pub config_file_path: Path,

    pub dirty: bool,
    pub dirty_file_path: Path,

    pub command_line: Option<ParsedCommandLine>,
    command_line_with_typings_files: Mutex<Option<ParsedCommandLine>>,
    command_line_with_typings_files_init: OnceLock<()>,

    pub program: Option<Arc<Program>>,
    pub program_update_kind: ProgramUpdateKind,
    pub program_last_update: u64,

    pub potential_project_references: Option<HashSet<Path>>,
    pub typings_files: Vec<String>,
}

impl Project {
    pub fn new(config_file_name: String, kind: Kind, current_directory: String) -> Self {
        Project {
            kind,
            current_directory,
            config_file_name: config_file_name.clone(),
            config_file_path: Path(config_file_name),

            dirty: true,
            dirty_file_path: Path::default(),

            command_line: None,
            command_line_with_typings_files: Mutex::new(None),
            command_line_with_typings_files_init: OnceLock::new(),

            program: None,
            program_update_kind: ProgramUpdateKind::None,
            program_last_update: 0,

            potential_project_references: None,
            typings_files: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.config_file_name
    }

    pub fn display_name(&self, cwd: &str) -> String {
        if self.kind == Kind::Inferred {
            return crate::tspath::get_base_file_name(&self.current_directory);
        }

        let _ = cwd;
        self.config_file_name.clone()
    }

    pub fn id(&self) -> &Path {
        &self.config_file_path
    }

    pub fn config_file_name_str(&self) -> &str {
        if self.kind != Kind::Configured {
            panic!("ConfigFileName called on non-configured project");
        }
        &self.config_file_name
    }

    pub fn config_file_path(&self) -> &Path {
        if self.kind != Kind::Configured {
            panic!("ConfigFilePath called on non-configured project");
        }
        &self.config_file_path
    }

    pub fn get_program(&self) -> Option<&Arc<Program>> {
        self.program.as_ref()
    }

    pub fn has_file(&self, _file_name: &str) -> bool {
        todo!("Project::has_file requires program integration")
    }

    pub fn contains_file(&self, _path: &Path) -> bool {
        self.program.is_some()
    }

    pub fn is_source_from_project_reference(&self, _path: &Path) -> bool {
        false
    }

    pub fn clone_shallow(&self) -> Project {
        Project {
            kind: self.kind,
            current_directory: self.current_directory.clone(),
            config_file_name: self.config_file_name.clone(),
            config_file_path: self.config_file_path.clone(),
            dirty: self.dirty,
            dirty_file_path: self.dirty_file_path.clone(),
            command_line: self.command_line.clone(),
            command_line_with_typings_files: Mutex::new(None),
            command_line_with_typings_files_init: OnceLock::new(),
            program: self.program.clone(),
            program_update_kind: ProgramUpdateKind::None,
            program_last_update: self.program_last_update,
            potential_project_references: self.potential_project_references.clone(),
            typings_files: self.typings_files.clone(),
        }
    }

    pub fn set_command_line(&mut self, command_line: ParsedCommandLine) {
        self.command_line = Some(command_line);
        *self.command_line_with_typings_files.lock().unwrap() = None;

        self.command_line_with_typings_files_init = OnceLock::new();
        self.potential_project_references = None;
        self.dirty = true;
        self.dirty_file_path = Path::default();
    }

    pub fn create_program(&mut self) -> CreateProgramResult {
        todo!("Project::create_program requires full compiler integration")
    }
}
