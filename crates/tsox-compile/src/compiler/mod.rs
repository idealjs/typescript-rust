pub(crate) use std::collections::HashMap;
pub(crate) use std::sync::{Arc, Mutex};
pub(crate) use tsox_checker::binder::Binder;
pub(crate) use tsox_core::core::compiler_options::CompilerOptions;
pub(crate) use tsox_core::core::compiler_options::ModuleKind;
pub(crate) use tsox_core::core::compiler_options::ScriptTarget;
pub(crate) use tsox_core::core::text::TextRange;
pub(crate) use tsox_core::core::tristate::Tristate;
pub(crate) use tsox_core::diagnostics::Category;
pub(crate) use tsox_frontend::ast::NodeSymbolMap;
pub(crate) use tsox_frontend::ast::ScriptKind;
pub(crate) use tsox_frontend::ast::SourceFile;
pub(crate) use tsox_frontend::ast::diagnostic::Diagnostic;
pub(crate) use tsox_frontend::parser::Parser;
pub(crate) use tsox_frontend::parser::script_kind_from_file_name;
pub(crate) use tsox_tsoptions::tsoptions::ParsedCommandLine;
pub(crate) use tsox_tsoptions::vfs::FS;
pub(crate) mod compiler_host_2;
pub(crate) mod extract_reference_types_directives;
pub(crate) mod impl_chunk;
pub(crate) mod import_resolution_mode_override;
pub(crate) mod package_dedupe;
pub(crate) mod program_2;
pub(crate) mod reference_path_directives;
pub(crate) mod program_3;
#[allow(unused_imports)]
pub use compiler_host_2::*;
#[allow(unused_imports)]
pub use extract_reference_types_directives::*;
#[allow(unused_imports)]
pub use impl_chunk::*;
#[allow(unused_imports)]
pub use import_resolution_mode_override::*;
#[allow(unused_imports)]
pub use program_2::*;
#[allow(unused_imports)]
pub use reference_path_directives::*;
#[allow(unused_imports)]
pub use program_3::*;
#[cfg(test)]
pub(crate) mod tests;
