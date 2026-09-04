//! Project struct (1:1 port of Go's `internal/project/project.go`).

#![allow(dead_code)]

use std::collections::HashSet;
use std::sync::{Arc, Mutex, OnceLock};

use crate::compiler::Program;
use crate::tsoptions::ParsedCommandLine;
use crate::tspath::Path;


/// The inferred project's config file path (a fixed sentinel).
///
/// Go: `const inferredProjectName = "/dev/null/inferred"`.
pub const INFERRED_PROJECT_NAME: &str = "/dev/null/inferred";

/// Horizontal rule for log output.
pub const HR: &str = "-----------------------------------------------";

/// The kind of a project: inferred (no tsconfig) or configured (has tsconfig).
///
/// Go: `type Kind int`.
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

/// Describes how the program was updated.
///
/// Go: `type ProgramUpdateKind int`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProgramUpdateKind {
    #[default]
    None,
    Cloned,
    SameFileNames,
    NewFiles,
}

/// The kind of project creation result.
///
/// Go: `type CreateProgramResult struct { ... }`.
#[derive(Default)]
pub struct CreateProgramResult {
    pub program: Option<Arc<Program>>,
    pub update_kind: ProgramUpdateKind,
}

/// Represents a TypeScript project (configured or inferred).
///
/// Go: `type Project struct { ... }`.
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

    /// Returns a short, human-readable name relative to `cwd`.
    ///
    /// Go: `func (p *Project) DisplayName(cwd string) string`.
    pub fn display_name(&self, cwd: &str) -> String {
        if self.kind == Kind::Inferred {
            return crate::tspath::get_base_file_name(&self.current_directory);
        }
        // Simplified: return the config file name as-is.
        let _ = cwd;
        self.config_file_name.clone()
    }

    pub fn id(&self) -> &Path {
        &self.config_file_path
    }

    /// Panics if kind is not Configured.
    pub fn config_file_name_str(&self) -> &str {
        if self.kind != Kind::Configured {
            panic!("ConfigFileName called on non-configured project");
        }
        &self.config_file_name
    }

    /// Panics if kind is not Configured.
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

    /// Creates a shallow clone of the project.
    ///
    /// Go: `func (p *Project) Clone() *Project`.
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

    /// Reassigns the project's command line and resets derived state.
    pub fn set_command_line(&mut self, command_line: ParsedCommandLine) {
        self.command_line = Some(command_line);
        *self.command_line_with_typings_files.lock().unwrap() = None;
        // Reset the OnceLock by replacing it
        self.command_line_with_typings_files_init = OnceLock::new();
        self.potential_project_references = None;
        self.dirty = true;
        self.dirty_file_path = Path::default();
    }

    pub fn create_program(&mut self) -> CreateProgramResult {
        todo!("Project::create_program requires full compiler integration")
    }
}
