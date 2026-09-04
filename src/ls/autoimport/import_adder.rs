//! Import statement adder (1:1 port of Go's `internal/ls/autoimport/import_adder.go`).

use std::collections::HashMap;
use std::sync::Arc;

use crate::ast::{Node, Symbol};
use crate::checker::Checker;
use crate::compiler::Program;
use crate::ls::lsutil::format_code_options::FormatCodeSettings;
use crate::ls::lsutil::user_preferences::UserPreferences;
use crate::lsp::lsproto::TextEdit;

use super::fix::Fix;
use super::view::View;
use super::AddAsTypeOnly;

/// Trait for accumulating import fixes and producing text edits.
///
/// Mirrors `autoimport.ImportAdder` interface in Go.
pub trait ImportAdderTrait {
    fn has_fixes(&self) -> bool;
    fn add_import_from_exported_symbol(
        &mut self,
        symbol: &Symbol,
        is_valid_type_only_use_site: bool,
    );
    fn add_import_fix(&mut self, fix: &Fix);
    fn edits(&mut self) -> Vec<TextEdit>;
}

/// Tracks modifications to an existing import clause or binding pattern.
///
/// Mirrors `autoimport.addToExistingState` in Go.
#[derive(Debug, Default)]
pub struct AddToExistingState {
    pub import_clause_or_binding_pattern: Option<Arc<Node>>,
    pub default_import: Option<super::fix::NewImportBinding>,
    pub named_imports: HashMap<String, super::fix::NewImportBinding>,
}

/// Tracks new imports for a given module specifier.
///
/// Mirrors `autoimport.importsCollection` in Go.
#[derive(Debug, Default)]
pub struct ImportsCollection {
    pub default_import: Option<super::fix::NewImportBinding>,
    pub named_imports: HashMap<String, super::fix::NewImportBinding>,
    pub namespace_like_import: Option<super::fix::NewImportBinding>,
    pub use_require: bool,
}

/// Creates a new-imports map key from module specifier and type-only flag.
///
/// Mirrors `newImportsKey` in Go.
pub fn new_imports_key(module_specifier: &str, top_level_type_only: bool) -> String {
    if top_level_type_only {
        format!("1|{}", module_specifier)
    } else {
        format!("0|{}", module_specifier)
    }
}

/// The concrete import adder.
///
/// Mirrors `autoimport.importAdder` in Go.
pub struct ImportAdder {
    // Context
    pub checker: Option<Arc<Checker>>,
    pub view: Option<Arc<View>>,
    pub format_options: FormatCodeSettings,
    pub preferences: UserPreferences,

    // State
    pub add_to_namespace: Vec<Fix>,
    pub import_type: Vec<Fix>,
    pub add_to_existing: HashMap<*mut Node, AddToExistingState>,
    pub new_imports: HashMap<String, ImportsCollection>,
}

impl ImportAdder {
    /// Creates a new import adder.
    ///
    /// Mirrors `NewImportAdder` in Go.
    pub fn new(
        _program: &Program,
        checker: Arc<Checker>,
        _file: &crate::ast::SourceFile,
        view: Arc<View>,
        format_options: FormatCodeSettings,
        _converters: (),
        preferences: UserPreferences,
    ) -> ImportAdder {
        ImportAdder {
            checker: Some(checker),
            view: Some(view),
            format_options,
            preferences,
            add_to_namespace: Vec::new(),
            import_type: Vec::new(),
            add_to_existing: HashMap::new(),
            new_imports: HashMap::new(),
        }
    }

    /// Whether any fixes have been accumulated.
    pub fn has_fixes(&self) -> bool {
        !self.add_to_namespace.is_empty()
            || !self.import_type.is_empty()
            || !self.add_to_existing.is_empty()
            || !self.new_imports.is_empty()
    }
}

impl ImportAdderTrait for ImportAdder {
    fn has_fixes(&self) -> bool {
        ImportAdder::has_fixes(self)
    }

    fn add_import_from_exported_symbol(
        &mut self,
        _symbol: &Symbol,
        _is_valid_type_only_use_site: bool,
    ) {
        // Requires checker.GetMergedSymbol, checker.SkipAlias, SymbolToExport — stubbed.
        todo!("add_import_from_exported_symbol requires checker and export resolution")
    }

    fn add_import_fix(&mut self, _fix: &Fix) {
        // The Go logic is a large switch on fix.Kind with complex state management.
        // Requires change.Tracker, lsutil helpers — stubbed.
        todo!("add_import_fix requires change.Tracker infrastructure")
    }

    fn edits(&mut self) -> Vec<TextEdit> {
        // Requires change.NewTracker, lsutil.GetQuotePreference, insertImports — stubbed.
        todo!("edits requires change.Tracker infrastructure")
    }
}

/// Reduces two `AddAsTypeOnly` values, taking the maximum.
///
/// Mirrors `reduceAddAsTypeOnlyValues` in Go.
pub fn reduce_add_as_type_only_values(prev: AddAsTypeOnly, new: AddAsTypeOnly) -> AddAsTypeOnly {
    if new > prev { new } else { prev }
}

/// Gets the name for an exported symbol.
///
/// Mirrors `getNameForExportedSymbol` in Go.
pub fn get_name_for_exported_symbol(_symbol: &Symbol, _prefer_capitalized: bool) -> String {
    todo!("get_name_for_exported_symbol requires getDefaultLikeExportNameFromDeclaration")
}

/// Converts a type to an auto-importable type node.
///
/// Mirrors `TypeToAutoImportableTypeNode` in Go.
pub fn type_to_auto_importable_type_node(
    _c: &Checker,
    _import_adder: &mut dyn ImportAdderTrait,
    _t: &crate::checker::Type,
    _context_node: &Node,
) -> Option<Arc<Node>> {
    todo!("type_to_auto_importable_type_node requires checker.TypeToTypeNode")
}
