use super::flags::{CheckFlags, SymbolFlags};
use crate::ast::node::Node;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug)]
pub struct Symbol {
    pub flags: SymbolFlags,
    pub check_flags: CheckFlags,
    pub name: String,
    pub declarations: Vec<Arc<Node>>,
    pub value_declaration: Option<Arc<Node>>,
    pub members: SymbolTable,
    pub exports: SymbolTable,
    pub parent: Option<Arc<Symbol>>,
    pub export_symbol: Option<Arc<Symbol>>,
    id: AtomicU64,
}

impl Symbol {
    pub fn new(flags: SymbolFlags, name: impl Into<String>) -> Self {
        Self {
            flags,
            check_flags: CheckFlags::None,
            name: name.into(),
            declarations: Vec::new(),
            value_declaration: None,
            members: SymbolTable::default(),
            exports: SymbolTable::default(),
            parent: None,
            export_symbol: None,
            id: AtomicU64::new(0),
        }
    }

    pub fn id(&self) -> u64 {
        let mut id = self.id.load(Ordering::Relaxed);
        if id == 0 {
            id = NEXT_SYMBOL_ID.fetch_add(1, Ordering::Relaxed);
            self.id.store(id, Ordering::Relaxed);
        }
        id
    }

    pub fn is_external_module(&self) -> bool {
        self.flags.contains(SymbolFlags::ValueModule) && self.name.starts_with('"')
    }

    pub fn is_static(&self) -> bool {
        false
    }

    pub fn combined_local_and_export_symbol_flags(&self) -> SymbolFlags {
        if let Some(export) = &self.export_symbol {
            self.flags | export.flags
        } else {
            self.flags
        }
    }
}

static NEXT_SYMBOL_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Default, Clone)]
pub struct SymbolTable {
    pub entries: HashMap<String, Arc<Symbol>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, name: &str) -> Option<&Arc<Symbol>> {
        self.entries.get(name)
    }

    pub fn insert(&mut self, name: impl Into<String>, symbol: Arc<Symbol>) {
        self.entries.insert(name.into(), symbol);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, String, Arc<Symbol>> {
        self.entries.iter()
    }
}
