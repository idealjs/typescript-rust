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

#[derive(Debug, Default)]
pub struct AddToExistingState {
    pub import_clause_or_binding_pattern: Option<Arc<Node>>,
    pub default_import: Option<super::fix::NewImportBinding>,
    pub named_imports: HashMap<String, super::fix::NewImportBinding>,
}

#[derive(Debug, Default)]
pub struct ImportsCollection {
    pub default_import: Option<super::fix::NewImportBinding>,
    pub named_imports: HashMap<String, super::fix::NewImportBinding>,
    pub namespace_like_import: Option<super::fix::NewImportBinding>,
    pub use_require: bool,
}

pub fn new_imports_key(module_specifier: &str, top_level_type_only: bool) -> String {
    if top_level_type_only {
        format!("1|{}", module_specifier)
    } else {
        format!("0|{}", module_specifier)
    }
}

pub struct ImportAdder {

    pub checker: Option<Arc<Checker>>,
    pub view: Option<Arc<View>>,
    pub format_options: FormatCodeSettings,
    pub preferences: UserPreferences,

    pub add_to_namespace: Vec<Fix>,
    pub import_type: Vec<Fix>,
    pub add_to_existing: HashMap<*mut Node, AddToExistingState>,
    pub new_imports: HashMap<String, ImportsCollection>,
}

impl ImportAdder {

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

        todo!("add_import_from_exported_symbol requires checker and export resolution")
    }

    fn add_import_fix(&mut self, _fix: &Fix) {

        todo!("add_import_fix requires change.Tracker infrastructure")
    }

    fn edits(&mut self) -> Vec<TextEdit> {

        todo!("edits requires change.Tracker infrastructure")
    }
}

pub fn reduce_add_as_type_only_values(prev: AddAsTypeOnly, new: AddAsTypeOnly) -> AddAsTypeOnly {
    if new > prev { new } else { prev }
}

pub fn get_name_for_exported_symbol(_symbol: &Symbol, _prefer_capitalized: bool) -> String {
    todo!("get_name_for_exported_symbol requires getDefaultLikeExportNameFromDeclaration")
}

pub fn type_to_auto_importable_type_node(
    _c: &Checker,
    _import_adder: &mut dyn ImportAdderTrait,
    _t: &crate::checker::Type,
    _context_node: &Node,
) -> Option<Arc<Node>> {
    todo!("type_to_auto_importable_type_node requires checker.TypeToTypeNode")
}
