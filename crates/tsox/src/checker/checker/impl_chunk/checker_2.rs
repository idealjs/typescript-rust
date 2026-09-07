#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn merge_global_symbols(dst: &Arc<Symbol>, src: &Arc<Symbol>) {
        let dst_mut = Arc::as_ptr(dst) as *mut Symbol;
        unsafe {
            (*dst_mut).flags |= src.flags;
            for d in &src.declarations {
                if !dst
                    .declarations
                    .iter()
                    .any(|existing| Arc::ptr_eq(existing, d))
                {
                    (*dst_mut).declarations.push(Arc::clone(d));
                }
            }
            if dst.value_declaration.is_none() {
                (*dst_mut).value_declaration = src.value_declaration.clone();
            }
            for (name, member) in src.members.entries.iter() {
                match (*dst_mut).members.entries.get(name) {
                    Some(existing) => Self::merge_global_symbols(existing, member),
                    None => {
                        (*dst_mut)
                            .members
                            .entries
                            .insert(name.clone(), Arc::clone(member));
                    }
                }
            }
            for (name, export) in src.exports.entries.iter() {
                match (*dst_mut).exports.entries.get(name) {
                    Some(existing) => Self::merge_global_symbols(existing, export),
                    None => {
                        (*dst_mut)
                            .exports
                            .entries
                            .insert(name.clone(), Arc::clone(export));
                    }
                }
            }
        }
    }

    pub(crate) fn populate_globals(&mut self) {
        for file in &self.files {
            if file.external_module_indicator.is_some() {
                continue;
            }

            let symbol_map = self.program.symbol_map();
            if let Some(file_sym) = symbol_map.symbol_of(&file.node) {
                for (name, sym) in file_sym.members.iter() {
                    match self.globals.get(name) {
                        Some(existing) => Self::merge_global_symbols(existing, sym),
                        None => {
                            self.globals.insert(name.clone(), Arc::clone(sym));
                        }
                    }
                }

                if let Some(locals) = symbol_map.locals_of(&file.node) {
                    for (name, sym) in locals.iter() {
                        match self.globals.get(name) {
                            Some(existing) => Self::merge_global_symbols(existing, sym),
                            None => {
                                self.globals.insert(name.clone(), Arc::clone(sym));
                            }
                        }
                    }
                }
            }
        }

        for file in &self.files {
            for aug_name in &file.module_augmentations {
                let Some(module_node) = aug_name.parent.clone() else {
                    continue;
                };
                if !crate::ast::is_global_scope_augmentation(&module_node) {
                    continue;
                }
                let symbol_map = self.program.symbol_map();
                let mut aug_members: Vec<(String, Arc<Symbol>)> = Vec::new();
                if let Some(module_sym) = symbol_map.symbol_of(&module_node) {
                    aug_members.extend(
                        module_sym
                            .exports
                            .iter()
                            .map(|(k, v)| (k.clone(), Arc::clone(v))),
                    );
                    aug_members.extend(
                        module_sym
                            .members
                            .iter()
                            .map(|(k, v)| (k.clone(), Arc::clone(v))),
                    );
                }
                if let Some(locals) = symbol_map.locals_of(&module_node) {
                    aug_members.extend(locals.iter().map(|(k, v)| (k.clone(), Arc::clone(v))));
                }
                for (name, sym) in aug_members {
                    match self.globals.get(&name) {
                        Some(existing) => Self::merge_global_symbols(existing, &sym),
                        None => {
                            self.globals.insert(name, sym);
                        }
                    }
                }
            }
        }

        self.ensure_host_globals();

        self.ensure_jsx_namespace();

        self.report_missing_global_types();
    }

    pub(crate) fn report_missing_global_types(&mut self) {
        const GLOBAL_TYPE_NAMES: &[&str] = &[
            "Array",
            "Boolean",
            "CallableFunction",
            "Function",
            "IArguments",
            "NewableFunction",
            "Number",
            "Object",
            "RegExp",
            "String",
        ];
        for name in GLOBAL_TYPE_NAMES {
            if self.globals.get(*name).is_none() {
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    None,
                    crate::core::text::TextRange::default(),
                    crate::diagnostics::messages_generated::CANNOT_FIND_GLOBAL_TYPE_0,
                    vec![(*name).to_string()],
                ));
            }
        }
    }
}
