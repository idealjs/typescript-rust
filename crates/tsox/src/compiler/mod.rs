use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::ast::NodeSymbolMap;
use crate::ast::ScriptKind;
use crate::ast::SourceFile;
use crate::ast::diagnostic::Diagnostic;
use crate::ast::{self};
use crate::binder::Binder;
use crate::core::compiler_options::{
    CompilerOptions, ModuleKind, ModuleResolutionKind, ScriptTarget,
};
use crate::core::text::TextRange;
use crate::core::tristate::Tristate;
use crate::diagnostics::Category;
use crate::module;
use crate::parser::{Parser, script_kind_from_file_name};
use crate::tspath;
use crate::vfs::FS;
use crate::tsoptions::ParsedCommandLine;
mod compiler_host_2;
mod program_2;
mod program_3;
mod import_resolution_mode_override;
mod extract_reference_types_directives;
mod impl_chunk;
#[allow(unused_imports)]
pub use compiler_host_2::*;
#[allow(unused_imports)]
pub use program_2::*;
#[allow(unused_imports)]
pub use program_3::*;
#[allow(unused_imports)]
pub use import_resolution_mode_override::*;
#[allow(unused_imports)]
pub use extract_reference_types_directives::*;
#[allow(unused_imports)]
pub use impl_chunk::*;
#[cfg(test)]
mod tests;
