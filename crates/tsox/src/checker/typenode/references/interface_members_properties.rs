#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn add_property_signature_member(
        &mut self,
        member: &Arc<Node>,
        symbol_table: &mut SymbolTable,
        props: &mut Vec<Arc<Symbol>>,
    ) {
        let NodeData::PropertySignatureDeclaration(data) = &member.data else {
            unreachable!()
        };
                let name = self.get_property_name_from_node(&data.name);
                if name.is_empty() {
            return;
                }
                let mut prop_type = self.get_type_from_type_node(&data.type_node);
                let is_optional = data
                    .postfix_token
                    .as_ref()
                    .map(|t| t.kind == SyntaxKind::QuestionToken)
                    .unwrap_or(false);

                if is_optional {
                    prop_type = self.get_optional_type(prop_type);
                }
                let mut flags = SymbolFlags::Property;
                if is_optional {
                    flags |= SymbolFlags::Optional;
                }
                let mut symbol = Symbol::new(flags, name.clone());

                symbol.declarations = vec![Arc::clone(member)];

                if let Some(m) = &data.modifiers {
                    if m.modifier_flags.contains(ModifierFlags::Readonly) {
                        symbol.check_flags |= CheckFlags::Readonly;
                    }
                }
                let symbol = Arc::new(symbol);
                self.value_symbol_links.insert(
                    &symbol,
                    ValueSymbolLinks {
                        resolved_type: Some(prop_type),
                        ..Default::default()
                    },
                );
                symbol_table.insert(name, Arc::clone(&symbol));
                props.push(symbol);
    }

    pub(crate) fn add_method_signature_member(
        &mut self,
        member: &Arc<Node>,
        symbol_table: &mut SymbolTable,
        props: &mut Vec<Arc<Symbol>>,
    ) {
        let NodeData::MethodSignatureDeclaration(data) = &member.data else {
            unreachable!()
        };
                let name = self.get_property_name_from_node(&data.name);
                if name.is_empty() {
            return;
                }

                self.push_scope(member);
                let return_type = match data.type_node.as_ref() {
                    Some(tn) => self.get_type_from_type_node(tn),
                    None => self.get_any_type(),
                };
                let sig = self.build_signature_from_function_like_type_node(
                    &data.parameters,
                    return_type,
                    false,
                    None,
                    Some(Arc::clone(member)),
                );
                self.pop_scope();

                if let Some(existing) = symbol_table.get(&name).cloned() {
                    let existing_type = self
                        .value_symbol_links
                        .get(&existing)
                        .and_then(|l| l.resolved_type.clone());
                    let merged_sigs = existing_type
                        .as_ref()
                        .and_then(|t| t.as_structured().map(|s| s.call_signatures().to_vec()))
                        .unwrap_or_default();
                    let mut all_sigs = merged_sigs;
                    all_sigs.push(sig);
                    let fn_type = self.create_function_or_constructor_type(all_sigs, false);
                    self.value_symbol_links.insert(
                        &existing,
                        ValueSymbolLinks {
                            resolved_type: Some(fn_type),
                            ..Default::default()
                        },
                    );
            return;
                }
                let fn_type = self.create_function_or_constructor_type(vec![sig], false);
                let symbol = Arc::new(Symbol::new(SymbolFlags::Property, name.clone()));
                self.value_symbol_links.insert(
                    &symbol,
                    ValueSymbolLinks {
                        resolved_type: Some(fn_type),
                        ..Default::default()
                    },
                );
                symbol_table.insert(name, Arc::clone(&symbol));
                props.push(symbol);
    }

    pub(crate) fn add_property_declaration_member(
        &mut self,
        member: &Arc<Node>,
        symbol_table: &mut SymbolTable,
        props: &mut Vec<Arc<Symbol>>,
    ) {
        let NodeData::PropertyDeclaration(data) = &member.data else {
            unreachable!()
        };
                if is_static_modifier(&data.modifiers) {
            return;
                }
                let name = self.get_property_name_from_node(&data.name);
                if name.is_empty() {
            return;
                }
                let mut prop_type =
                    match data.type_node.as_ref() {
                        Some(tn) => self.get_type_from_type_node(tn),
                        None => match data.initializer.as_ref() {
                            Some(init) => {
                                let raw =
                                    match &init.data {
                                        NodeData::Identifier(_) => {
                                            match self.resolve_identifier(init) {
                                        Some(sym) if sym.flags.intersects(
                                            SymbolFlags::BlockScopedVariable
                                                | SymbolFlags::FunctionScopedVariable,
                                        ) => self.get_type_of_symbol(&sym),
                                        _ => self.get_type_of_node(init),
                                    }
                                        }
                                        _ => self.get_type_of_node(init),
                                    };
                                let is_readonly = data.modifiers.as_ref().is_some_and(|m| {
                                    m.modifier_flags.contains(ModifierFlags::Readonly)
                                });
                                let widened = if is_readonly {
                                    raw
                                } else if self.is_empty_array_literal(init) {
                                    if self.strict_null_checks {
                                        self.get_widened_literal_type(&raw)
                                    } else {
                                        self.create_array_type(self.get_any_type())
                                    }
                                } else {
                                    self.get_widened_literal_type(&raw)
                                };
                                let regularized =
                                    self.get_regular_type_of_literal_type(&widened);
                                self.widen_initializer_type(&regularized)
                            }
                            None => self.get_any_type(),
                        },
                    };
                let is_optional = data
                    .postfix_token
                    .as_ref()
                    .map(|t| t.kind == SyntaxKind::QuestionToken)
                    .unwrap_or(false);
                if is_optional {
                    prop_type = self.get_optional_type(prop_type);
                }
                let mut flags = SymbolFlags::Property;
                if is_optional {
                    flags |= SymbolFlags::Optional;
                }
                let mut symbol = Symbol::new(flags, name.clone());

                symbol.declarations.push(Arc::clone(member));

                if let Some(m) = &data.modifiers {
                    if m.modifier_flags.contains(ModifierFlags::Readonly) {
                        symbol.check_flags |= CheckFlags::Readonly;
                    }
                }
                let symbol = Arc::new(symbol);
                self.value_symbol_links.insert(
                    &symbol,
                    ValueSymbolLinks {
                        resolved_type: Some(prop_type),
                        ..Default::default()
                    },
                );
                symbol_table.insert(name, Arc::clone(&symbol));
                props.push(symbol);
    }

    pub(crate) fn add_method_declaration_member(
        &mut self,
        member: &Arc<Node>,
        symbol_table: &mut SymbolTable,
        props: &mut Vec<Arc<Symbol>>,
    ) {
        let NodeData::MethodDeclaration(data) = &member.data else {
            unreachable!()
        };
                if is_static_modifier(&data.modifiers) {
            return;
                }
                let name = self.get_property_name_from_node(&data.name);
                if name.is_empty() {
            return;
                }

                self.push_scope(member);
                let return_type = match data.type_node.as_ref() {
                    Some(tn) => self.get_type_from_type_node(tn),
                    None => self.get_any_type(),
                };
                let sig = self.build_signature_from_function_like_type_node(
                    &data.parameters,
                    return_type,
                    false,
                    None,
                    Some(Arc::clone(member)),
                );
                self.pop_scope();

                if let Some(existing) = symbol_table.get(&name).cloned() {
                    if data.body.is_some() {
                        let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                        unsafe {
                            (*existing_mut).declarations.push(Arc::clone(member));
                        }
            return;
                    }
                    let existing_type = self
                        .value_symbol_links
                        .get(&existing)
                        .and_then(|l| l.resolved_type.clone());
                    let merged_sigs = existing_type
                        .as_ref()
                        .and_then(|t| t.as_structured().map(|s| s.call_signatures().to_vec()))
                        .unwrap_or_default();
                    let mut all_sigs = merged_sigs;
                    all_sigs.push(sig);
                    let fn_type = self.create_function_or_constructor_type(all_sigs, false);
                    self.value_symbol_links.insert(
                        &existing,
                        ValueSymbolLinks {
                            resolved_type: Some(fn_type),
                            ..Default::default()
                        },
                    );
            return;
                }
                let fn_type = self.create_function_or_constructor_type(vec![sig], false);
                let mut symbol = Symbol::new(SymbolFlags::Property, name.clone());

                symbol.declarations.push(Arc::clone(member));
                let symbol = Arc::new(symbol);
                self.value_symbol_links.insert(
                    &symbol,
                    ValueSymbolLinks {
                        resolved_type: Some(fn_type),
                        ..Default::default()
                    },
                );
                symbol_table.insert(name, Arc::clone(&symbol));
                props.push(symbol);
    }
}