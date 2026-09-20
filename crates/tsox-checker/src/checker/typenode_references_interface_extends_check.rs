#![allow(unused_imports)]

use crate::checker::typenode_references::*;

impl Checker {
    pub(crate) fn structured_types_identical(
        &mut self,
        a: &Arc<Type>,
        b: &Arc<Type>,
        depth: u32,
    ) -> bool {
        if Arc::ptr_eq(a, b) {
            return true;
        }
        if a.flags != b.flags {
            return false;
        }
        if a.flags.contains(TYPE_FLAGS_SINGLETON) {
            return true;
        }
        if let (TypeData::Literal(la), TypeData::Literal(lb)) = (&a.data, &b.data) {
            return la.value == lb.value;
        }
        if let (Some(sa), Some(sb)) = (a.symbol.as_ref(), b.symbol.as_ref())
            && Arc::ptr_eq(sa, sb)
        {
            return true;
        }
        if !matches!(
            (&a.data, &b.data),
            (TypeData::Object(_), TypeData::Object(_))
                | (TypeData::Interface(_), TypeData::Interface(_))
        ) {
            return self.is_type_identical_to(a, b);
        }
        if depth > 6 {
            return self.type_to_string(a) == self.type_to_string(b);
        }
        for kind in [SignatureKind::Call, SignatureKind::Construct] {
            let sigs_a = self.get_signatures_of_type(a, kind);
            let sigs_b = self.get_signatures_of_type(b, kind);
            if sigs_a.len() != sigs_b.len() {
                return false;
            }
            for (sa, sb) in sigs_a.iter().zip(sigs_b.iter()) {
                if sa.parameters.len() != sb.parameters.len() {
                    return false;
                }
                for (pa, pb) in sa.parameters.iter().zip(sb.parameters.iter()) {
                    let ta = self.get_type_of_symbol(pa);
                    let tb = self.get_type_of_symbol(pb);
                    if !self.structured_types_identical(&ta, &tb, depth + 1) {
                        return false;
                    }
                }
                let ra = self.get_non_circular_return_type_of_signature(sa);
                let rb = self.get_non_circular_return_type_of_signature(sb);
                if !self.structured_types_identical(&ra, &rb, depth + 1) {
                    return false;
                }
            }
        }
        let pa = a
            .as_structured()
            .map(|s| s.properties.clone())
            .unwrap_or_default();
        let pb = b
            .as_structured()
            .map(|s| s.properties.clone())
            .unwrap_or_default();
        if pa.len() != pb.len() {
            return false;
        }
        for (x, y) in pa.iter().zip(pb.iter()) {
            if x.name != y.name
                || x.flags.contains(tsox_frontend::ast::SymbolFlags::Optional)
                    != y.flags.contains(tsox_frontend::ast::SymbolFlags::Optional)
            {
                return false;
            }
            let tx = self.get_type_of_symbol(x);
            let ty = self.get_type_of_symbol(y);
            if !self.structured_types_identical(&tx, &ty, depth + 1) {
                return false;
            }
        }
        true
    }

    pub(crate) fn report_interface_simultaneous_extends(
        &mut self,
        symbol: &Arc<Symbol>,
        interface_decls: &[Arc<Node>],
        own_result: &Arc<Type>,
        base_types: &[(Arc<Node>, Arc<Type>)],
    ) {
        if base_types.len() < 2 {
            return;
        }
        let sym_key = Arc::as_ptr(symbol) as *const tsox_frontend::ast::Symbol;
        if !self.interface_simultaneous_reported.insert(sym_key) {
            return;
        }
        let name_loc = interface_decls.first().and_then(|d| match &d.data {
            NodeData::InterfaceDeclaration(d) => Some(d.name.loc),
            _ => None,
        });
        let Some(name_loc) = name_loc else {
            return;
        };
        let own_structured = match &own_result.data {
            TypeData::Object(o) => Some(&o.structured),
            _ => None,
        };
        let Some(own) = own_structured else {
            return;
        };

        let mut seen: Vec<(String, Arc<tsox_frontend::ast::Symbol>, Option<Arc<Type>>)> = Vec::new();
        for prop in &own.properties {
            if prop.name.is_empty() || prop.name.starts_with('[') {
                continue;
            }
            seen.push((prop.name.clone(), Arc::clone(prop), None));
        }
        let interface_name = symbol.name.clone();
        for (_, base) in base_types {
            let Some(base_structured) = base.as_structured() else {
                continue;
            };
            for prop in &base_structured.properties {
                if prop.name.is_empty() || prop.name.starts_with('[') {
                    continue;
                }
                match seen
                    .iter()
                    .find(|(name, _, _)| name == &prop.name)
                    .cloned()
                {
                    None => {
                        seen.push((prop.name.clone(), Arc::clone(prop), Some(Arc::clone(base))));
                    }
                    Some((_, existing_prop, Some(existing_base))) => {
                        let existing_type = self.get_type_of_symbol(&existing_prop);
                        let prop_type = self.get_type_of_symbol(prop);
                        if self.structured_types_identical(&existing_type, &prop_type, 0) {
                            continue;
                        }
                        let type_name1 = self.type_to_string(&existing_base);
                        let type_name2 = self.type_to_string(base);
                        let file = self.current_file.clone();
                        let mut diag = tsox_frontend::ast::Diagnostic::new(
                            file,
                            name_loc,
                            tsox_core::diagnostics::messages_generated::
                                INTERFACE_0_CANNOT_SIMULTANEOUSLY_EXTEND_TYPES_1_AND_2,
                            vec![
                                interface_name.clone(),
                                type_name1.clone(),
                                type_name2.clone(),
                            ],
                        );
                        diag.message_chain = vec![tsox_frontend::ast::Diagnostic::new(
                            None,
                            name_loc,
                            tsox_core::diagnostics::messages_generated::
                                NAMED_PROPERTY_0_OF_TYPES_1_AND_2_ARE_NOT_IDENTICAL,
                            vec![prop.name.clone(), type_name1, type_name2],
                        )];
                        self.diagnostics.add(diag);
                    }
                    Some((_, _, None)) => {}
                }
            }
        }
    }

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
                    Arc::as_ptr(symbol) as *const tsox_frontend::ast::Symbol,
                    Arc::as_ptr(type_ref_node) as *const tsox_frontend::ast::Node,
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
                            let derived_display = {
                                let tps = interface_decls.iter().find_map(|d| {
                                    match &d.data {
                                        NodeData::InterfaceDeclaration(d) => {
                                            d.type_parameters.clone()
                                        }
                                        _ => None,
                                    }
                                });
                                self.generic_display_name(&symbol.name, tps.as_ref())
                            };
                            let base_display =
                                self.node_source_text(type_ref_node).unwrap_or_else(|| {
                                    base
                                        .symbol
                                        .as_ref()
                                        .map(|s| s.name.clone())
                                        .unwrap_or_default()
                                });
                            let file = self.current_file.clone();
                            let mut diag = tsox_frontend::ast::Diagnostic::new(
                                        file,
                                        name_loc,
                                        tsox_core::diagnostics::messages_generated::
                                            INTERFACE_0_INCORRECTLY_EXTENDS_INTERFACE_1,
                                        vec![derived_display, base_display],
                                    );

                            let dt_str = self.type_to_string(&dt);
                            let bt_str = self.type_to_string(&bt);
                            self.relater_error_chain = captured;
                            self.relater_chain_active = true;
                            self.push_relation_head_with_tp_note(
                                        &dt,
                                        &bt,
                                        tsox_core::diagnostics::messages_generated::
                                            TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                                        vec![dt_str, bt_str],
                                    );
                            self.relater_report_error(
                                        tsox_core::diagnostics::messages_generated::
                                            TYPES_OF_PROPERTY_0_ARE_INCOMPATIBLE,
                                        vec![self.chain_property_arg_name(own_prop)],
                                    );
                            let entries = std::mem::take(&mut self.relater_error_chain);
                            self.relater_chain_active = was_active;

                            let mut child: Option<tsox_frontend::ast::Diagnostic> = None;
                            for entry in entries
                                .iter()
                                .filter(|e| !e.message.elided_in_compatibility_pyramid)
                            {
                                let mut d = tsox_frontend::ast::Diagnostic::new(
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
