#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn add_index_signature_member(
        &mut self,
        member: &Arc<Node>,
        index_infos: &mut Vec<Arc<crate::checker::IndexInfo>>,
    ) {
        let NodeData::IndexSignatureDeclaration(data) = &member.data else {
            unreachable!()
        };
                let mut key_type = None;
                let value_type;
                if let Some(param) = data.parameters.iter().next() {
                    if let NodeData::ParameterDeclaration(pd) = &param.data {
                        key_type = pd
                            .type_node
                            .as_ref()
                            .map(|t| self.get_type_from_type_node(t));
                    }
                }
                value_type = Some(self.get_type_from_type_node(&data.type_node));
                let is_readonly = member
                    .modifiers()
                    .as_ref()
                    .is_some_and(|m| m.flags().contains(ModifierFlags::Readonly));
                index_infos.push(Arc::new(crate::checker::IndexInfo {
                    key_type,
                    value_type,
                    is_readonly,
                    declaration: Some(Arc::clone(member)),
                    index_symbol: None,
                    components: Vec::new(),
                }));
    }

    pub(crate) fn add_get_accessor_member(
        &mut self,
        member: &Arc<Node>,
        symbol_table: &mut SymbolTable,
        props: &mut Vec<Arc<Symbol>>,
    ) {
        let NodeData::GetAccessorDeclaration(data) = &member.data else {
            unreachable!()
        };
                if is_static_modifier(&data.modifiers) {
            return;
                }
                let name = self.get_property_name_from_node(&data.name);
                if name.is_empty() {
            return;
                }

                let prop_type = match data.type_node.as_ref() {
                    Some(tn) => self.get_type_from_type_node(tn),
                    None => self.get_any_type(),
                };
                match symbol_table.get(&name).cloned() {
                    Some(existing) => {
                        let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                        unsafe {
                            (*existing_mut).flags |= SymbolFlags::GetAccessor;
                            (*existing_mut).declarations.push(Arc::clone(member));
                        }
                        self.value_symbol_links.insert(
                            &existing,
                            ValueSymbolLinks {
                                resolved_type: Some(prop_type),
                                ..Default::default()
                            },
                        );
                    }
                    None => {
                        let mut symbol = Symbol::new(
                            SymbolFlags::Property | SymbolFlags::GetAccessor,
                            name.clone(),
                        );
                        symbol.declarations.push(Arc::clone(member));
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
                }
    }

    pub(crate) fn add_set_accessor_member(
        &mut self,
        member: &Arc<Node>,
        symbol_table: &mut SymbolTable,
        props: &mut Vec<Arc<Symbol>>,
    ) {
        let NodeData::SetAccessorDeclaration(data) = &member.data else {
            unreachable!()
        };
                if is_static_modifier(&data.modifiers) {
            return;
                }
                let name = self.get_property_name_from_node(&data.name);
                if name.is_empty() {
            return;
                }

                let prop_type = data
                    .parameters
                    .iter()
                    .next()
                    .and_then(|p| {
                        if let NodeData::ParameterDeclaration(pd) = &p.data {
                            pd.type_node
                                .as_ref()
                                .map(|tn| self.get_type_from_type_node(tn))
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| self.get_any_type());
                match symbol_table.get(&name).cloned() {
                    Some(existing) => {
                        let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                        unsafe {
                            (*existing_mut).flags |= SymbolFlags::SetAccessor;
                            (*existing_mut).declarations.push(Arc::clone(member));
                        }
                    }
                    None => {
                        let mut symbol = Symbol::new(
                            SymbolFlags::Property | SymbolFlags::SetAccessor,
                            name.clone(),
                        );
                        symbol.declarations.push(Arc::clone(member));
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
                }
    }

    pub(crate) fn add_call_signature_member(
        &mut self,
        member: &Arc<Node>,
        call_signatures: &mut Vec<Arc<Signature>>,
    ) {
        let NodeData::CallSignatureDeclaration(data) = &member.data else {
            unreachable!()
        };
                let suppress = self
                    .current_file
                    .as_ref()
                    .is_some_and(|f| f.file_name.starts_with("bundled://"));
                if suppress {
                    self.push_ts2304_suppression();
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
                if suppress {
                    self.pop_ts2304_suppression();
                }
                call_signatures.push(sig);
    }

    pub(crate) fn add_construct_signature_member(
        &mut self,
        member: &Arc<Node>,
        construct_signatures: &mut Vec<Arc<Signature>>,
    ) {
        let NodeData::ConstructSignatureDeclaration(data) = &member.data else {
            unreachable!()
        };
                let suppress = self
                    .current_file
                    .as_ref()
                    .is_some_and(|f| f.file_name.starts_with("bundled://"));
                if suppress {
                    self.push_ts2304_suppression();
                }

                self.push_scope(member);
                let return_type = match data.type_node.as_ref() {
                    Some(tn) => self.get_type_from_type_node(tn),
                    None => self.get_any_type(),
                };
                let sig = self.build_signature_from_function_like_type_node(
                    &data.parameters,
                    return_type,
                    true,
                    None,
                    Some(Arc::clone(member)),
                );
                self.pop_scope();
                if suppress {
                    self.pop_ts2304_suppression();
                }
                construct_signatures.push(sig);
    }

    pub(crate) fn add_constructor_properties(
        &mut self,
        member: &Arc<Node>,
        symbol_table: &mut SymbolTable,
        props: &mut Vec<Arc<Symbol>>,
    ) {
        let NodeData::ConstructorDeclaration(data) = &member.data else {
            unreachable!()
        };
                for param in data.parameters.iter() {
                    let NodeData::ParameterDeclaration(pd) = &param.data else {
                        continue;
                    };
                    if pd.name.kind != SyntaxKind::Identifier {
                        continue;
                    }
                    let Some(modifiers) = &pd.modifiers else {
                        continue;
                    };
                    if !modifiers.modifier_flags.intersects(
                        ModifierFlags::Public
                            | ModifierFlags::Private
                            | ModifierFlags::Protected
                            | ModifierFlags::Readonly,
                    ) {
                        continue;
                    }
                    let name = pd.name.text().to_string();
                    if name.is_empty() || symbol_table.get(&name).is_some() {
                        continue;
                    }
                    let prop_type = match pd.type_node.as_ref() {
                        Some(tn) => self.get_type_from_type_node(tn),
                        None => match pd.initializer.as_ref() {
                            Some(init) => self.get_type_of_node(init),
                            None => self.get_any_type(),
                        },
                    };
                    let mut symbol = Symbol::new(SymbolFlags::Property, name.clone());

                    symbol.declarations.push(Arc::clone(param));
                    if modifiers.modifier_flags.contains(ModifierFlags::Readonly) {
                        symbol.check_flags |= CheckFlags::Readonly;
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
    }
}