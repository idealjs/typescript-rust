use std::sync::Arc;
use std::sync::atomic::AtomicI32;

use crate::ast::{Node, SourceFile, Symbol, SymbolFlags};
use crate::checker::Checker;
use crate::module::Resolver;
use crate::tspath;

use super::export::{Export, ExportSyntax, ModuleID};
use super::{
    INTERNAL_SYMBOL_NAME_DEFAULT, INTERNAL_SYMBOL_NAME_EXPORT_EQUALS,
    INTERNAL_SYMBOL_NAME_EXPORT_STAR,
};

#[derive(Debug, Default)]
pub struct ExtractorStats {
    pub exports: AtomicI32,
    pub used_checker: AtomicI32,
}

pub struct SymbolExtractor {
    pub package_name: String,
    pub stats: Arc<ExtractorStats>,
    pub checker: Option<Arc<Checker>>,
    pub to_path: Option<Box<dyn Fn(&str) -> tspath::Path + Send + Sync>>,

    pub realpath: Option<Box<dyn Fn(&str) -> String + Send + Sync>>,
}

pub struct ExportExtractor {
    pub symbol_extractor: SymbolExtractor,
    pub module_resolver: Option<Arc<Resolver>>,
}

impl ExportExtractor {

    pub fn stats(&self) -> &ExtractorStats {
        &self.symbol_extractor.stats
    }
}

pub struct CheckerLease {
    pub used: bool,
    pub checker: Arc<Checker>,
}

impl CheckerLease {
    pub fn new(checker: Arc<Checker>) -> Self {
        CheckerLease {
            used: false,
            checker,
        }
    }

    pub fn get_checker(&mut self) -> &Checker {
        self.used = true;
        &self.checker
    }

    pub fn try_checker(&self) -> Option<&Checker> {
        if self.used { Some(&self.checker) } else { None }
    }
}

pub fn new_symbol_extractor(
    package_name: &str,
    checker: Arc<Checker>,
    to_path: Option<Box<dyn Fn(&str) -> tspath::Path + Send + Sync>>,
    realpath: Option<Box<dyn Fn(&str) -> String + Send + Sync>>,
) -> SymbolExtractor {
    SymbolExtractor {
        package_name: package_name.to_string(),
        stats: Arc::new(ExtractorStats::default()),
        checker: Some(checker),
        to_path,
        realpath,
    }
}

impl SymbolExtractor {

    pub fn get_module_id(&self, file: &SourceFile) -> ModuleID {

        if let (Some(realpath_fn), Some(to_path_fn)) = (&self.realpath, &self.to_path) {
            let rp = realpath_fn(&file.file_name);
            return to_path_fn(&rp).0;
        }

        file.file_name.clone()
    }

    pub fn get_module_id_for_symbol(&self, _symbol: &Symbol) -> Option<(ModuleID, bool)> {
        todo!("get_module_id_for_symbol requires tryGetModuleIDAndFileNameOfModuleSymbol")
    }
}

impl ExportExtractor {

    pub fn extract_from_file(&self, _file: &SourceFile) -> Vec<Export> {
        todo!("extract_from_file requires ast.Symbol, file.Symbol.Exports iteration")
    }

    pub fn extract_from_module(&self, _file: &SourceFile) -> Vec<Export> {
        todo!("extract_from_module requires module augmentation parsing")
    }

    pub fn extract_from_module_declaration(
        &self,
        _decl: &Node,
        _file: &SourceFile,
        _module_id: ModuleID,
        _module_file_name: &str,
        _exports: &mut Vec<Export>,
    ) {
        todo!("extract_from_module_declaration requires decl.Symbol.Exports")
    }
}

impl SymbolExtractor {

    pub fn extract_from_symbol(
        &self,
        _name: &str,
        _symbol: &Symbol,
        _module_id: ModuleID,
        _module_file_name: &str,
        _file: &SourceFile,
        _exports: &mut Vec<Export>,
    ) {
        todo!("extract_from_symbol requires checker.GetExportsOfModule and createExport")
    }

    pub fn create_export(
        &self,
        _symbol: &Symbol,
        _module_id: ModuleID,
        _module_file_name: &str,
        _syntax: ExportSyntax,
        _file: &SourceFile,
        _checker_lease: &mut CheckerLease,
    ) -> (Option<Export>, Option<Arc<Symbol>>) {
        todo!("create_export requires checker and lsutil helpers")
    }
}

pub fn should_ignore_symbol(symbol: &Symbol) -> bool {
    symbol.flags.contains(SymbolFlags::Prototype)
}

pub fn get_syntax(_symbol: &Symbol) -> ExportSyntax {
    todo!("get_syntax requires ast declaration kind inspection")
}

pub fn is_unusable_name(name: &str) -> bool {
    name.is_empty()
        || name == "_default"
        || name == INTERNAL_SYMBOL_NAME_EXPORT_STAR
        || name == INTERNAL_SYMBOL_NAME_DEFAULT
        || name == INTERNAL_SYMBOL_NAME_EXPORT_EQUALS
}

pub fn file_name_for_default_export_name(
    _target_symbol: Option<&Symbol>,
    module_file_name: &str,
    module_id: &ModuleID,
) -> String {

    if !module_file_name.is_empty() {
        module_file_name.to_string()
    } else {
        module_id.clone()
    }
}
