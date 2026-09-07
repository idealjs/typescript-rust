#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn push_interface_type_argument_mapping(
        &mut self,
        interface_decls: &[Arc<Node>],
        tp_symbols: &[Arc<Symbol>],
        arg_types: &[Arc<Type>],
    ) {
        let mut mapping = HashMap::new();
        for (i, tp_sym) in tp_symbols.iter().enumerate() {
            if let Some(arg) = arg_types.get(i) {
                let k = Arc::as_ptr(tp_sym) as *const crate::ast::Symbol;
                mapping.insert(k, Arc::clone(arg));
            }
        }
        for decl in interface_decls {
            let NodeData::InterfaceDeclaration(d) = &decl.data else {
                continue;
            };
            let Some(tps) = &d.type_parameters else {
                continue;
            };
            let sym_map = self.program.symbol_map();
            for (i, tp) in tps.iter().enumerate() {
                let Some(tp_sym) = sym_map.symbol_of(tp) else {
                    continue;
                };

                let idx = if let Some(first_sym) = tp_symbols.get(i) {
                    if first_sym.name == tp_sym.name {
                        i
                    } else {
                        tp_symbols
                            .iter()
                            .position(|s| s.name == tp_sym.name)
                            .unwrap_or(i)
                    }
                } else {
                    i
                };
                if let Some(arg) = arg_types.get(idx) {
                    let k = Arc::as_ptr(tp_sym) as *const crate::ast::Symbol;
                    mapping.insert(k, Arc::clone(arg));
                }
            }
        }
        self.type_argument_stack.push(mapping);
    }

    pub(crate) fn collect_interface_base_types(
        &mut self,
        interface_decls: &[Arc<Node>],
        heritage_degraded: &mut bool,
    ) -> Vec<(Arc<Node>, Arc<Type>)> {
        let mut base_types: Vec<(Arc<Node>, Arc<Type>)> = Vec::new();
        for decl in interface_decls {
            if let NodeData::InterfaceDeclaration(d) = &decl.data {
                if let Some(heritage) = &d.heritage_clauses {
                    for clause in heritage.iter() {
                        if let NodeData::HeritageClause(hc) = &clause.data
                            && hc.token == SyntaxKind::ExtendsKeyword
                        {
                            for type_ref in hc.types.iter() {
                                let bt = self.get_type_from_type_node(type_ref);

                                if crate::checker::utilities::is_type_error(&bt)
                                    && !*heritage_degraded
                                {
                                    *heritage_degraded = true;
                                    self.heritage_degraded_events += 1;
                                }
                                base_types.push((Arc::clone(type_ref), bt));
                            }
                        }
                    }
                }
            }
        }
        base_types
    }
}
