use crate::ast::{Node, Symbol, SymbolFlags};
use crate::checker::Checker;
use crate::ls::lsutil::symbol_display::{ScriptElementKind, ScriptElementKindModifier};
use crate::tspath;

use super::{INTERNAL_SYMBOL_NAME_DEFAULT, INTERNAL_SYMBOL_NAME_EXPORT_EQUALS};

pub type ModuleID = String;

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct ExportID {
    pub module_id: ModuleID,
    pub export_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum ExportSyntax {
    #[default]
    None,

    Modifier,

    Named,

    DefaultModifier,

    DefaultDeclaration,

    Equals,

    UMD,

    Star,

    CommonJSModuleExports,

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

#[derive(Debug, Clone)]
pub struct Export {
    pub export_id: ExportID,
    pub module_file_name: String,
    pub syntax: ExportSyntax,
    pub flags: SymbolFlags,
    pub local_name: String,

    pub through: String,

    pub target: ExportID,
    pub is_type_only: bool,
    pub script_element_kind: ScriptElementKind,
    pub script_element_kind_modifiers: ScriptElementKindModifier,

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

pub fn symbol_to_export(_symbol: &Symbol, _ch: &Checker) -> Option<Export> {
    todo!("symbol_to_export requires checker.IsExternalModuleSymbol and ast helpers")
}

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

fn extract_first_export(
    _symbol: &Symbol,
    _ch: &Checker,
    _module_id: ModuleID,
    _module_file_name: &str,
    _file: &Node,
) -> Option<Export> {
    todo!("extractFirstExport requires symbolExtractor (see extract.rs)")
}
