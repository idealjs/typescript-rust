//! Export kind enum + helpers (1:1 port of Go's `internal/ls/autoimport/export.go`).

use crate::ast::{Node, Symbol, SymbolFlags};
use crate::checker::Checker;
use crate::ls::lsutil::symbol_display::{ScriptElementKind, ScriptElementKindModifier};
use crate::tspath;

use super::{INTERNAL_SYMBOL_NAME_DEFAULT, INTERNAL_SYMBOL_NAME_EXPORT_EQUALS};

/// Uniquely identifies a module across multiple declarations.
/// If the export is from an ambient module declaration, this is the module name.
/// If the export is from a module augmentation, this is the `Path()` of the resolved module file.
/// Otherwise this is the `Path()` of the exporting source file.
pub type ModuleID = String;

/// An export identifier: module ID + export name.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct ExportID {
    pub module_id: ModuleID,
    pub export_name: String,
}

/// The syntax form used to export a symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum ExportSyntax {
    #[default]
    None,
    /// `export const x = {}`
    Modifier,
    /// `export { x }`
    Named,
    /// `export default function f() {}`
    DefaultModifier,
    /// `export default f`
    DefaultDeclaration,
    /// `export = x`
    Equals,
    /// `export as namespace x`
    UMD,
    /// `export * from "module"`
    Star,
    /// `module.exports = {}`
    CommonJSModuleExports,
    /// `exports.x = {}`
    CommonJSExportsProperty,
}

impl ExportSyntax {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExportSyntax::None => "None",
            ExportSyntax::Modifier => "Modifier",
            ExportSyntax::Named => "Named",
            ExportSyntax::DefaultModifier => "DefaultModifier",
            ExportSyntax::DefaultDeclaration => "DefaultDeclaration",
            ExportSyntax::Equals => "Equals",
            ExportSyntax::UMD => "UMD",
            ExportSyntax::Star => "Star",
            ExportSyntax::CommonJSModuleExports => "CommonJSModuleExports",
            ExportSyntax::CommonJSExportsProperty => "CommonJSExportsProperty",
        }
    }
}

/// An export entry in the auto-import index.
///
/// Mirrors `autoimport.Export` in Go.
#[derive(Debug, Clone)]
pub struct Export {
    pub export_id: ExportID,
    pub module_file_name: String,
    pub syntax: ExportSyntax,
    pub flags: SymbolFlags,
    pub local_name: String,
    /// The name of the module symbol's export that this export was found through,
    /// either `export=`, `InternalSymbolNameExportStar`, or empty string.
    pub through: String,

    // Checker-set fields
    pub target: ExportID,
    pub is_type_only: bool,
    pub script_element_kind: ScriptElementKind,
    pub script_element_kind_modifiers: ScriptElementKindModifier,

    /// The file where the export was found.
    pub path: tspath::Path,
    pub package_name: String,
}

impl Default for Export {
    fn default() -> Self {
        Self {
            export_id: ExportID::default(),
            module_file_name: String::new(),
            syntax: ExportSyntax::None,
            flags: SymbolFlags::None,
            local_name: String::new(),
            through: String::new(),
            target: ExportID::default(),
            is_type_only: false,
            script_element_kind: ScriptElementKind::default(),
            script_element_kind_modifiers: ScriptElementKindModifier::default(),
            path: tspath::Path(String::new()),
            package_name: String::new(),
        }
    }
}

impl Export {
    pub fn name(&self) -> &str {
        if !self.local_name.is_empty() {
            &self.local_name
        } else if self.export_id.export_name == INTERNAL_SYMBOL_NAME_EXPORT_EQUALS {
            &self.target.export_name
        } else {
            &self.export_id.export_name
        }
    }

    pub fn is_renameable(&self) -> bool {
        self.export_id.export_name == INTERNAL_SYMBOL_NAME_EXPORT_EQUALS
            || self.export_id.export_name == INTERNAL_SYMBOL_NAME_DEFAULT
    }

    pub fn ambient_module_name(&self) -> &str {
        if !tspath::is_external_module_name_relative(&self.export_id.module_id) {
            &self.export_id.module_id
        } else {
            ""
        }
    }

    pub fn is_unresolved_alias(&self) -> bool {
        self.flags == SymbolFlags::Alias
    }
}

impl super::index::Named for Export {
    fn name(&self) -> &str {
        Export::name(self)
    }
}

/// Converts a symbol to an Export entry.
///
/// Mirrors `autoimport.SymbolToExport` in Go.
pub fn symbol_to_export(_symbol: &Symbol, _ch: &Checker) -> Option<Export> {
    // Requires checker, ast navigation, and module-symbol helpers not yet ported.
    todo!("symbol_to_export requires checker.IsExternalModuleSymbol and ast helpers")
}

/// Tries to get a module export matching the given target symbol.
///
/// Mirrors `autoimport.tryGetModuleExport` in Go.
fn try_get_module_export(
    _export_name: &str,
    _target: &Symbol,
    _module_symbol: &Symbol,
    _ch: &Checker,
    _module_id: ModuleID,
    _module_file_name: &str,
    _file: &Node,
) -> Option<Export> {
    todo!("try_get_module_export requires checker.TryGetMemberInModuleExportsAndProperties")
}

/// Extracts the first export from a symbol.
///
/// Mirrors `autoimport.extractFirstExport` in Go.
fn extract_first_export(
    _symbol: &Symbol,
    _ch: &Checker,
    _module_id: ModuleID,
    _module_file_name: &str,
    _file: &Node,
) -> Option<Export> {
    todo!("extractFirstExport requires symbolExtractor (see extract.rs)")
}
