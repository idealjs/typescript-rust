#![allow(unused_imports)]

use super::*;

pub type SymbolTableId = u64;

pub(crate) const ST_KIND_SHIFT: u32 = 61;

pub(crate) const ST_KIND_LOCALS: SymbolTableId = 0 << ST_KIND_SHIFT;
pub(crate) const ST_KIND_EXPORTS: SymbolTableId = 1 << ST_KIND_SHIFT;
pub(crate) const ST_KIND_MEMBERS: SymbolTableId = 2 << ST_KIND_SHIFT;
pub(crate) const ST_KIND_GLOBALS: SymbolTableId = 3 << ST_KIND_SHIFT;
pub(crate) const ST_KIND_RESOLVED_EXPORTS: SymbolTableId = 4 << ST_KIND_SHIFT;

pub(crate) const ST_KIND_MASK: SymbolTableId = 0x7 << ST_KIND_SHIFT;

pub(crate) fn symbol_table_id_from_locals(node: &Node) -> SymbolTableId {
    ST_KIND_LOCALS | node.id()
}

pub(crate) fn symbol_table_id_from_exports(sym: &Symbol) -> SymbolTableId {
    ST_KIND_EXPORTS | sym.id()
}

pub(crate) fn symbol_table_id_from_resolved_exports(sym: &Symbol) -> SymbolTableId {
    ST_KIND_RESOLVED_EXPORTS | sym.id()
}

pub(crate) fn symbol_table_id_from_members(sym: &Symbol) -> SymbolTableId {
    ST_KIND_MEMBERS | sym.id()
}

pub(crate) fn symbol_table_id_from_globals() -> SymbolTableId {
    ST_KIND_GLOBALS
}

pub struct AccessibleSymbolChainContext {
    pub symbol: Arc<Symbol>,
    pub enclosing_declaration: Option<Arc<Node>>,
    pub meaning: SymbolFlags,
    pub use_only_external_aliasing: bool,

    pub visited_symbol_tables_map: RefCell<HashMap<u64, HashMap<SymbolTableId, ()>>>,
}

impl Clone for AccessibleSymbolChainContext {
    fn clone(&self) -> Self {
        Self {
            symbol: Arc::clone(&self.symbol),
            enclosing_declaration: self.enclosing_declaration.clone(),
            meaning: self.meaning,
            use_only_external_aliasing: self.use_only_external_aliasing,
            visited_symbol_tables_map: RefCell::new(
                self.visited_symbol_tables_map.borrow().clone(),
            ),
        }
    }
}

pub(crate) struct SymbolTableInScope {
    pub(crate) table: SymbolTable,
    pub(crate) table_id: SymbolTableId,
    pub(crate) ignore_qualification: bool,
    pub(crate) is_local_name_lookup: bool,
    pub(crate) scope_node: Option<Arc<Node>>,
}

pub(crate) fn has_non_global_augmentation_external_module_symbol(declaration: &Arc<Node>) -> bool {
    declaration.kind == SyntaxKind::ModuleDeclaration
}

pub(crate) fn has_external_module_symbol(declaration: &Arc<Node>) -> bool {
    declaration.kind == SyntaxKind::ModuleDeclaration
        || (declaration.kind == SyntaxKind::SourceFile)
}

pub(crate) fn get_qualified_left_meaning(right_meaning: SymbolFlags) -> SymbolFlags {
    if right_meaning == SymbolFlags::VALUE {
        SymbolFlags::VALUE
    } else {
        SymbolFlags::NAMESPACE
    }
}

pub(crate) fn is_property_or_method_declaration_symbol(symbol: &Symbol) -> bool {
    if !symbol.declarations.is_empty() {
        for declaration in &symbol.declarations {
            match declaration.kind {
                SyntaxKind::PropertyDeclaration
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor => continue,
                _ => return false,
            }
        }
        true
    } else {
        false
    }
}

pub(crate) fn is_umd_export_symbol(symbol: &Symbol) -> bool {
    !symbol.declarations.is_empty()
        && symbol
            .declarations
            .first()
            .map(|d| d.kind == SyntaxKind::NamespaceExportDeclaration)
            .unwrap_or(false)
}

pub(crate) fn is_namespace_reexport_declaration(node: &Arc<Node>) -> bool {
    node.kind == SyntaxKind::NamespaceExport
}
