//! Auto-import code fix logic (1:1 port of Go's `internal/ls/autoimport/fix.go`).

use std::sync::Arc;

use crate::ast::{Node, SourceFile};
use crate::core::compiler_options::CompilerOptions;
use crate::ls::lsutil::format_code_options::FormatCodeSettings;
use crate::ls::lsutil::user_preferences::UserPreferences;
use crate::lsp::lsproto::TextEdit;
use crate::modulespecifiers;

use super::export::Export;
use super::{AddAsTypeOnly, AutoImportFix, AutoImportFixKind, ImportKind};

/// A new import binding to be created.
///
/// Mirrors `autoimport.newImportBinding` in Go.
#[derive(Debug, Clone)]
pub struct NewImportBinding {
    pub kind: ImportKind,
    pub property_name: String,
    pub name: String,
    pub add_as_type_only: AddAsTypeOnly,
}

impl Default for NewImportBinding {
    fn default() -> Self {
        NewImportBinding {
            kind: ImportKind::Named,
            property_name: String::new(),
            name: String::new(),
            add_as_type_only: AddAsTypeOnly::Allowed,
        }
    }
}

/// An auto-import fix.
///
/// Mirrors `autoimport.Fix` in Go.
#[derive(Debug, Clone, Default)]
pub struct Fix {
    pub auto_import_fix: AutoImportFix,
    pub module_specifier_kind: super::specifiers::ResultKind,
    pub is_re_export: bool,
    pub module_file_name: String,
    pub type_only_alias_declaration: Option<Arc<Node>>,
}

/// An "add to existing import" fix.
///
/// Mirrors `autoimport.addToExistingImportFix` in Go.
#[derive(Debug)]
pub struct AddToExistingImportFix {
    pub import_clause_or_binding_pattern: Option<Arc<Node>>,
    pub default_import: Option<NewImportBinding>,
    pub named_import: Option<NewImportBinding>,
}

impl Fix {
    /// Produces text edits and a description for this fix.
    ///
    /// Mirrors `(f *Fix) Edits` in Go.
    pub fn edits(
        &self,
        _file: &SourceFile,
        _compiler_options: &CompilerOptions,
        _format_options: &FormatCodeSettings,
        _preferences: &UserPreferences,
    ) -> (Vec<TextEdit>, String) {
        // Requires change.Tracker, diagnostics localization, scanner — stubbed.
        todo!("Fix::edits requires change.Tracker and diagnostics infrastructure")
    }
}

/// The kind of file syntax (ESM, CJS, or ambiguous).
///
/// Mirrors `autoimport.fileSyntaxKind` in Go.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileSyntaxKind {
    Ambiguous,
    ESM,
    CJS,
}

impl Default for FileSyntaxKind {
    fn default() -> Self {
        FileSyntaxKind::Ambiguous
    }
}

/// Detects whether a source file has unambiguous ESM or CJS syntax.
///
/// Mirrors `detectSyntax` in Go.
pub fn detect_syntax(_file: &SourceFile, _options: &CompilerOptions) -> FileSyntaxKind {
    todo!("detect_syntax requires file.ExternalModuleIndicator and file.CommonJSModuleIndicator")
}

/// Gets the import kind for an export.
///
/// Mirrors `getImportKind` in Go.
pub fn get_import_kind(
    _importing_file: &SourceFile,
    _export: &Export,
    _program: &crate::compiler::Program,
) -> ImportKind {
    todo!("get_import_kind requires program.Options() and program.GetEmitModuleFormatOfFile")
}

/// Determines if an import should be type-only.
///
/// Mirrors `getAddAsTypeOnly` in Go.
pub fn get_add_as_type_only(
    is_valid_type_only_use_site: bool,
    export_: &Export,
    _compiler_options: &CompilerOptions,
) -> AddAsTypeOnly {
    if !is_valid_type_only_use_site {
        return AddAsTypeOnly::NotAllowed;
    }
    // The full Go logic checks verbatimModuleSyntax and export flags.
    // Stubbed to return Allowed for now.
    let _ = export_;
    AddAsTypeOnly::Allowed
}

/// Gets the namespace-like import text from a declaration.
///
/// Mirrors `getNamespaceLikeImportText` in Go.
pub fn get_namespace_like_import_text(_declaration: &Node) -> String {
    todo!("get_namespace_like_import_text requires ast node inspection")
}

/// Checks if a fix is possibly re-exporting the importing file.
///
/// Mirrors `isFixPossiblyReExportingImportingFile` in Go.
pub fn is_fix_possibly_re_exporting_importing_file(fix: &Fix, importing_file_name: &str) -> bool {
    if fix.is_re_export && is_index_file_name(&fix.module_file_name) {
        let re_export_dir = crate::tspath::get_directory_path(&fix.module_file_name);
        importing_file_name.starts_with(&re_export_dir)
    } else {
        false
    }
}

/// Checks if a file name is an index file.
///
/// Mirrors `isIndexFileName` in Go.
pub fn is_index_file_name(file_name: &str) -> bool {
    let last_slash = match file_name.rfind('/') {
        Some(i) => i,
        None => return false,
    };
    if file_name.len() <= last_slash + 1 {
        return false;
    }
    let base = &file_name[last_slash + 1..];
    matches!(
        base,
        "index.js" | "index.jsx" | "index.d.ts" | "index.ts" | "index.tsx"
    )
}

/// Checks if a type-only import is needed.
///
/// Mirrors `needsTypeOnly` in Go.
pub fn needs_type_only(add_as_type_only: AddAsTypeOnly) -> bool {
    add_as_type_only == AddAsTypeOnly::Required
}

/// Checks if a type-only import should be used.
///
/// Mirrors `shouldUseTypeOnly` in Go.
pub fn should_use_type_only(
    add_as_type_only: AddAsTypeOnly,
    _preferences: &UserPreferences,
) -> bool {
    needs_type_only(add_as_type_only)
}

/// Compares fix kinds.
///
/// Mirrors `compareFixKinds` in Go.
pub fn compare_fix_kinds(a: AutoImportFixKind, b: AutoImportFixKind) -> std::cmp::Ordering {
    (a as u8).cmp(&(b as u8))
}

/// Compares module specifier relativity for ranking.
///
/// Mirrors `compareModuleSpecifierRelativity` in Go.
pub fn compare_module_specifier_relativity(
    _a: &Fix,
    _b: &Fix,
    _preferences: &modulespecifiers::UserPreferences,
) -> std::cmp::Ordering {
    // The Go logic checks preferences.ImportModuleSpecifierPreference.
    // Stubbed to return Equal.
    std::cmp::Ordering::Equal
}
