use std::sync::Arc;

use crate::lsp::lsproto::Position;
use tsox_compile::compiler::Program;
use tsox_core::collections::multimap::MultiMap;
use tsox_core::collections::set::Set;
use tsox_core::core::tristate::Tristate;
use tsox_frontend::ast::SourceFile;

use crate::ls::autoimport::ModuleSpecifierEnding;
use crate::ls::autoimport_export::Export;
use crate::ls::autoimport_export::ExportID;
use crate::ls::autoimport_fix::Fix;
use crate::ls::autoimport_registry::Registry;

pub struct View {
    pub registry: Arc<Registry>,
    pub importing_file: Arc<SourceFile>,
    pub program: Arc<Program>,
    pub preferences: tsox_tsoptions::modulespecifiers::UserPreferences,
    pub project_key: tsox_core::tspath::Path,

    pub allowed_endings: Option<Vec<ModuleSpecifierEnding>>,
    pub conditions: Set<String>,
    pub should_use_uri_style_node_core_modules: Tristate,
    pub existing_imports: Option<MultiMap<crate::ls::autoimport_export::ModuleID, ExistingImport>>,
    pub should_use_require_for_fixes: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryKind {
    WordPrefix,
    ExactMatch,
    CaseInsensitiveMatch,
}

#[derive(Debug, Clone)]
pub struct ExistingImport {
    pub node: Arc<tsox_frontend::ast::Node>,
    pub module_specifier: String,
    pub index: usize,
}

impl PartialEq for ExistingImport {
    fn eq(&self, other: &Self) -> bool {
        self.module_specifier == other.module_specifier && self.index == other.index
    }
}

#[derive(Debug)]
pub struct FixAndExport {
    pub fix: Fix,
    pub export_: Export,
}

impl View {
    pub fn new(
        _registry: Arc<Registry>,
        _importing_file: Arc<SourceFile>,
        _project_key: tsox_core::tspath::Path,
        _program: Arc<Program>,
        _preferences: tsox_tsoptions::modulespecifiers::UserPreferences,
    ) -> View {
        todo!("View::new requires program and module infrastructure")
    }

    pub fn get_allowed_endings(&mut self) -> &[ModuleSpecifierEnding] {
        todo!("View::get_allowed_endings requires modulespecifiers infrastructure")
    }

    pub fn search(&self, _query: &str, _kind: QueryKind) -> Vec<Export> {
        todo!("View::search requires registry buckets")
    }

    pub fn search_by_export_id(&self, _id: &ExportID) -> Vec<Export> {
        todo!("View::search_by_export_id requires registry buckets")
    }

    pub fn get_completions(
        &self,
        _prefix: &str,
        _position: Position,
        _for_jsx: bool,
        _is_type_only_location: bool,
    ) -> Vec<FixAndExport> {
        todo!("View::get_completions requires scanner, checker, and fix infrastructure")
    }

    pub fn get_fixes(
        &self,
        _export: &Export,
        _for_jsx: bool,
        _is_valid_type_only_use_site: bool,
        _usage_position: Option<&Position>,
    ) -> Vec<Fix> {
        todo!("View::get_fixes requires fix infrastructure")
    }

    pub fn compare_fixes_for_ranking(&self, _a: &Fix, _b: &Fix) -> std::cmp::Ordering {
        todo!("View::compare_fixes_for_ranking")
    }

    pub fn compare_fixes_for_sorting(&self, _a: &Fix, _b: &Fix) -> std::cmp::Ordering {
        todo!("View::compare_fixes_for_sorting")
    }

    pub fn should_use_require(&mut self) -> bool {
        if let Some(v) = self.should_use_require_for_fixes {
            return v;
        }
        let v = self.compute_should_use_require();
        self.should_use_require_for_fixes = Some(v);
        v
    }

    fn compute_should_use_require(&self) -> bool {
        todo!("View::compute_should_use_require requires program infrastructure")
    }
}
