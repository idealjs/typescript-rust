#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn get_type_parameter_from_symbol(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        if let Some(links) = self.type_alias_links.get(symbol) {
            if let Some(ref t) = links.declared_type {
                return Arc::clone(t);
            }
        }
        let sym_key = Arc::as_ptr(symbol) as usize;
        if !self.type_parameter_resolving.insert(sym_key) {
            return Arc::new(Type {
                flags: TypeFlags::TypeParameter,
                object_flags: ObjectFlags::None,
                id: crate::checker::types::next_type_id(),
                symbol: Some(Arc::clone(symbol)),
                alias: None,
                data: TypeData::TypeParameter(TypeParameterData {
                    constrained: ConstrainedTypeData::default(),
                    constraint: None,
                    target: None,
                    mapper: None,
                    is_this_type: false,
                    resolved_default_type: OnceLock::new(),
                }),
            });
        }
        let mut constraint: Option<Arc<Type>> = None;
        for decl in &symbol.declarations {
            if let NodeData::TypeParameterDeclaration(data) = &decl.data {
                if let Some(constraint_node) = &data.constraint {
                    constraint = Some(self.get_type_from_type_node(constraint_node));
                }
                break;
            }
        }

        if let Some(c) = &constraint
            && self.constraint_chain_is_circular(sym_key, c)
        {
            if self.ts2313_reported.insert(sym_key) {
                let loc = symbol
                    .declarations
                    .iter()
                    .find_map(|d| match &d.data {
                        NodeData::TypeParameterDeclaration(td) => {
                            td.constraint.as_ref().map(|cn| cn.loc)
                        }
                        _ => None,
                    })
                    .or_else(|| symbol.declarations.first().map(|d| d.loc))
                    .unwrap_or_default();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    loc,
                    crate::diagnostics::messages_generated::
                        TYPE_PARAMETER_0_HAS_A_CIRCULAR_CONSTRAINT,
                    vec![symbol.name.clone()],
                ));
            }
            constraint = None;
        }
        let tp = Arc::new(Type {
            flags: TypeFlags::TypeParameter,
            object_flags: ObjectFlags::None,
            id: crate::checker::types::next_type_id(),
            symbol: Some(Arc::clone(symbol)),
            alias: None,
            data: TypeData::TypeParameter(TypeParameterData {
                constrained: ConstrainedTypeData::default(),
                constraint,
                target: None,
                mapper: None,
                is_this_type: false,
                resolved_default_type: OnceLock::new(),
            }),
        });
        self.type_alias_links.get_or_default(symbol).declared_type = Some(Arc::clone(&tp));
        self.type_parameter_resolving.remove(&sym_key);
        tp
    }

    pub(crate) fn constraint_chain_is_circular(
        &self,
        start_key: usize,
        constraint: &Arc<Type>,
    ) -> bool {
        let mut visited: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let mut current = constraint;
        for _ in 0..50 {
            let TypeData::TypeParameter(tp) = &current.data else {
                return false;
            };
            let Some(sym) = &current.symbol else {
                return false;
            };
            let key = Arc::as_ptr(sym) as usize;
            if !visited.insert(key) {
                return true;
            }
            match &tp.constraint {
                Some(next) => current = next,
                None => return key == start_key,
            }
        }
        false
    }
}
