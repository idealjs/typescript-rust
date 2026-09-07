#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn report_interface_extends_incompatibilities(
        &mut self,
        symbol: &Arc<Symbol>,
        interface_decls: &[Arc<Node>],
        own_result: &Arc<Type>,
        base_types: &[(Arc<Node>, Arc<Type>)],
    ) {
        let own_structured = match &own_result.data {
            TypeData::Object(o) => Some(&o.structured),
            _ => None,
        };
        let name_loc = interface_decls.first().and_then(|d| match &d.data {
            NodeData::InterfaceDeclaration(d) => Some(d.name.loc),
            _ => None,
        });
        if let (Some(own), Some(name_loc)) = (own_structured, name_loc) {
            for (type_ref_node, base) in base_types {
                let dedup_key = (
                    Arc::as_ptr(symbol) as *const crate::ast::Symbol,
                    Arc::as_ptr(type_ref_node) as *const crate::ast::Node,
                );
                if self.interface_extends_reported.contains(&dedup_key) {
                    continue;
                }
                let base_structured = match &base.data {
                    TypeData::Object(o) => Some(&o.structured),
                    _ => None,
                };
                let Some(base_structured) = base_structured else {
                    continue;
                };
                for own_prop in &own.properties {
                    let Some(base_prop) = base_structured.members.get(&own_prop.name) else {
                        continue;
                    };
                    let derived_type = self
                        .value_symbol_links
                        .get(own_prop)
                        .and_then(|l| l.resolved_type.clone());
                    let base_type = self
                        .value_symbol_links
                        .get(base_prop)
                        .and_then(|l| l.resolved_type.clone());
                    if let (Some(dt), Some(bt)) = (derived_type, base_type) {
                        let bt = match bt.symbol.as_ref() {
                            Some(bsym) => {
                                let tps = self.declared_type_parameter_types(bsym);
                                if !tps.is_empty()
                                    && bt.as_object().is_none_or(|o| o.type_arguments.is_empty())
                                {
                                    let anys: Vec<Arc<Type>> =
                                        std::iter::repeat(self.get_any_type())
                                            .take(tps.len())
                                            .collect();
                                    self.resolve_interface_type_ex(bsym, Some(anys))
                                } else {
                                    bt
                                }
                            }
                            None => bt,
                        };

                        let saved_chain = std::mem::take(&mut self.relater_error_chain);
                        let was_active = self.relater_chain_active;
                        self.relater_chain_active = true;
                        let incompatible = !self.is_type_assignable_to(&dt, &bt);
                        let captured =
                            std::mem::replace(&mut self.relater_error_chain, saved_chain);
                        self.relater_chain_active = was_active;
                        if incompatible {
                            self.interface_extends_reported.insert(dedup_key);
                            let base_name = base
                                .symbol
                                .as_ref()
                                .map(|s| s.name.clone())
                                .unwrap_or_default();
                            let file = self.current_file.clone();
                            let mut diag = crate::ast::Diagnostic::new(
                                        file,
                                        name_loc,
                                        crate::diagnostics::messages_generated::
                                            INTERFACE_0_INCORRECTLY_EXTENDS_INTERFACE_1,
                                        vec![symbol.name.clone(), base_name],
                                    );

                            let dt_str = self.type_to_string(&dt);
                            let bt_str = self.type_to_string(&bt);
                            self.relater_error_chain = captured;
                            self.relater_chain_active = true;
                            self.push_relation_head_with_tp_note(
                                        &dt,
                                        &bt,
                                        crate::diagnostics::messages_generated::
                                            TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                                        vec![dt_str, bt_str],
                                    );
                            self.relater_report_error(
                                        crate::diagnostics::messages_generated::
                                            TYPES_OF_PROPERTY_0_ARE_INCOMPATIBLE,
                                        vec![self.chain_property_arg_name(own_prop)],
                                    );
                            let entries = std::mem::take(&mut self.relater_error_chain);
                            self.relater_chain_active = was_active;

                            let mut child: Option<crate::ast::Diagnostic> = None;
                            for entry in entries
                                .iter()
                                .filter(|e| !e.message.elided_in_compatibility_pyramid)
                            {
                                let mut d = crate::ast::Diagnostic::new(
                                    None,
                                    name_loc,
                                    entry.message,
                                    entry.args.clone(),
                                );
                                if let Some(c) = child.take() {
                                    d.message_chain = vec![c];
                                }
                                child = Some(d);
                            }
                            if let Some(c) = child {
                                diag.message_chain = vec![c];
                            }
                            self.diagnostics.add(diag);
                            break;
                        }
                    }
                }
            }
        }
    }
}
