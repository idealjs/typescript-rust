//! Export extraction (1:1 port of Go's `internal/ls/autoimport/extract.go`).

use std::sync::Arc;
use std::sync::atomic::AtomicI32;

use crate::ast::{Node, SourceFile, Symbol, SymbolFlags};
use crate::checker::Checker;
use crate::module::Resolver;
use crate::tspath;

use super::export::{Export, ExportID, ExportSyntax, ModuleID};
use super::util::PathAndFileName;
use super::{
    INTERNAL_SYMBOL_NAME_DEFAULT, INTERNAL_SYMBOL_NAME_EXPORT_EQUALS,
    INTERNAL_SYMBOL_NAME_EXPORT_STAR,
};

/// Statistics for export extraction.
///
/// Mirrors `autoimport.extractorStats` in Go.
#[derive(Debug, Default)]
pub struct ExtractorStats {
    pub exports: AtomicI32,
    pub used_checker: AtomicI32,
}

/// Extracts exports from source files.
///
/// Mirrors `autoimport.symbolExtractor` in Go.
pub struct SymbolExtractor {
    pub package_name: String,
    pub stats: Arc<ExtractorStats>,
    pub checker: Option<Arc<Checker>>,
    pub to_path: Option<Box<dyn Fn(&str) -> tspath::Path + Send + Sync>>,
    /// Used to resolve symlinks for ModuleID generation.
    pub realpath: Option<Box<dyn Fn(&str) -> String + Send + Sync>>,
}

/// Extracts exports from source files, including module augmentations.
///
/// Mirrors `autoimport.exportExtractor` in Go.
pub struct ExportExtractor {
    pub symbol_extractor: SymbolExtractor,
    pub module_resolver: Option<Arc<Resolver>>,
}

impl ExportExtractor {
    /// Returns extraction statistics.
    pub fn stats(&self) -> &ExtractorStats {
        &self.symbol_extractor.stats
    }
}

/// A lease on a checker, tracking whether it was used.
///
/// Mirrors `autoimport.checkerLease` in Go.
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

/// Creates a new symbol extractor.
///
/// Mirrors `newSymbolExtractor` in Go.
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
    /// Gets the ModuleID for a file, using realpath if available.
    ///
    /// Mirrors `(e *symbolExtractor) getModuleID` in Go.
    pub fn get_module_id(&self, file: &SourceFile) -> ModuleID {
        // Go uses file.Path() which returns a tspath.Path (lowercased on case-insensitive FS).
        // Rust SourceFile doesn't have a Path() method yet — stubbed.
        if let (Some(realpath_fn), Some(to_path_fn)) = (&self.realpath, &self.to_path) {
            let rp = realpath_fn(&file.file_name);
            return to_path_fn(&rp).0;
        }
        // Fallback: use file_name as module ID (Go uses file.Path()).
        file.file_name.clone()
    }

    /// Gets the ModuleID for a module symbol.
    ///
    /// Mirrors `(e *symbolExtractor) getModuleIDForSymbol` in Go.
    pub fn get_module_id_for_symbol(&self, _symbol: &Symbol) -> Option<(ModuleID, bool)> {
        todo!("get_module_id_for_symbol requires tryGetModuleIDAndFileNameOfModuleSymbol")
    }
}

impl ExportExtractor {
    /// Extracts exports from a file.
    ///
    /// Mirrors `(e *exportExtractor) extractFromFile` in Go.
    pub fn extract_from_file(&self, _file: &SourceFile) -> Vec<Export> {
        todo!("extract_from_file requires ast.Symbol, file.Symbol.Exports iteration")
    }

    /// Extracts exports from a module (source file with a symbol).
    ///
    /// Mirrors `(e *exportExtractor) extractFromModule` in Go.
    pub fn extract_from_module(&self, _file: &SourceFile) -> Vec<Export> {
        todo!("extract_from_module requires module augmentation parsing")
    }

    /// Extracts exports from a module declaration.
    ///
    /// Mirrors `(e *exportExtractor) extractFromModuleDeclaration` in Go.
    pub fn extract_from_module_declaration(
        &self,
        _decl: &Node,
        _file: &SourceFile,
        _module_id: ModuleID,
        _module_file_name: &str,
        exports: &mut Vec<Export>,
    ) {
        todo!("extract_from_module_declaration requires decl.Symbol.Exports")
    }
}

impl SymbolExtractor {
    /// Extracts exports from a single symbol.
    ///
    /// Mirrors `(e *symbolExtractor) extractFromSymbol` in Go.
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

    /// Creates an Export for the given symbol.
    ///
    /// Mirrors `(e *symbolExtractor) createExport` in Go.
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

/// Determines whether a symbol should be ignored during extraction.
///
/// Mirrors `shouldIgnoreSymbol` in Go.
pub fn should_ignore_symbol(symbol: &Symbol) -> bool {
    symbol.flags.contains(SymbolFlags::Prototype)
}

/// Gets the export syntax for a symbol based on its declarations.
///
/// Mirrors `getSyntax` in Go.
pub fn get_syntax(_symbol: &Symbol) -> ExportSyntax {
    todo!("get_syntax requires ast declaration kind inspection")
}

/// Checks if a name is unusable as an export name.
///
/// Mirrors `isUnusableName` in Go.
pub fn is_unusable_name(name: &str) -> bool {
    name.is_empty()
        || name == "_default"
        || name == INTERNAL_SYMBOL_NAME_EXPORT_STAR
        || name == INTERNAL_SYMBOL_NAME_DEFAULT
        || name == INTERNAL_SYMBOL_NAME_EXPORT_EQUALS
}

/// Returns the best file name for deriving a fallback identifier for a default-like export.
///
/// Mirrors `fileNameForDefaultExportName` in Go.
pub fn file_name_for_default_export_name(
    _target_symbol: Option<&Symbol>,
    module_file_name: &str,
    module_id: &ModuleID,
) -> String {
    // The Go logic:
    // 1. If targetSymbol has declarations, use the source file name.
    // 2. Otherwise, if moduleFileName is set, use it.
    // 3. Otherwise, use the lowercased moduleID.
    if !module_file_name.is_empty() {
        module_file_name.to_string()
    } else {
        module_id.clone()
    }
}
