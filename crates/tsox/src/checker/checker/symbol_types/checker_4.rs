#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn build_type_of_class_declaration(
        &mut self,
        node: &Arc<Node>,
        members: &Arc<NodeList>,
    ) -> Arc<Type> {
        self.push_scope(node);

        let instance_type = self.build_class_instance_type_with_base(node);
        let mut construct_sigs: Vec<Arc<Signature>> = Vec::new();
        for member in members.iter() {
            if member.kind != SyntaxKind::Constructor {
                continue;
            }
            let params = match &member.data {
                crate::ast::NodeData::ConstructorDeclaration(data) => &data.parameters,
                _ => continue,
            };
            let sig = self.build_signature_from_function_like_type_node(
                params,
                Arc::clone(&instance_type),
                true,
                None,
                Some(Arc::clone(member)),
            );
            construct_sigs.push(sig);
        }
        self.pop_scope();
        if construct_sigs.is_empty() {
            let mut inherited: Option<(Arc<Node>, Arc<Node>)> = None;
            let mut cursor = Arc::clone(node);

            for _ in 0..1000 {
                let Some((base_node, _)) = self.extends_base_of(&cursor) else {
                    break;
                };
                if Arc::ptr_eq(&base_node, &cursor) {
                    break;
                }
                if let crate::ast::NodeData::ClassDeclaration(data) = &base_node.data {
                    if let Some(ctor) = data
                        .members
                        .iter()
                        .find(|m| matches!(m.data, crate::ast::NodeData::ConstructorDeclaration(_)))
                    {
                        inherited = Some((Arc::clone(ctor), Arc::clone(&base_node)));
                        break;
                    }
                }
                cursor = base_node;
            }
            if let Some((ctor_decl, _)) = inherited {
                if let crate::ast::NodeData::ConstructorDeclaration(data) = &ctor_decl.data {
                    let params = Arc::clone(&data.parameters);
                    let sig = self.build_signature_from_function_like_type_node(
                        &params,
                        Arc::clone(&instance_type),
                        true,
                        None,
                        Some(ctor_decl),
                    );
                    construct_sigs.push(sig);
                }
            }
        }
        if construct_sigs.is_empty() {
            let sig = self.build_signature_from_function_like_type_node(
                &Arc::new(NodeList::default()),
                Arc::clone(&instance_type),
                true,
                None,
                None,
            );
            construct_sigs.push(sig);
        }

        if node.has_syntactic_modifier(ModifierFlags::Abstract) {
            construct_sigs = construct_sigs
                .into_iter()
                .map(|sig| {
                    let s = crate::checker::types::Signature {
                        id: sig.id,
                        flags: sig.flags | crate::checker::types::SignatureFlags::Abstract,
                        min_argument_count: sig.min_argument_count,
                        resolved_min_argument_count: sig.resolved_min_argument_count,
                        declaration: sig.declaration.clone(),
                        type_parameters: sig.type_parameters.clone(),
                        parameters: sig.parameters.clone(),
                        this_parameter: sig.this_parameter.clone(),
                        resolved_return_type: std::sync::OnceLock::new(),
                        resolved_type_predicate: sig.resolved_type_predicate.clone(),
                        target: None,
                        mapper: sig.mapper.clone(),
                        isolated_signature_type: std::sync::OnceLock::new(),
                        instantiated_parameter_types: sig.instantiated_parameter_types.clone(),
                    };
                    if let Some(rt) = sig.resolved_return_type.get() {
                        let _ = s.resolved_return_type.set(rt.clone());
                    }
                    if let Some(it) = sig.isolated_signature_type.get() {
                        let _ = s.isolated_signature_type.set(it.clone());
                    }
                    Arc::new(s)
                })
                .collect();
        }
        let ctor_type = self.create_function_or_constructor_type(construct_sigs, true);

        self.attach_class_statics(&ctor_type, node);

        if let Some(class_sym) = self.program.symbol_map().symbol_of(node) {
            let t_mut = Arc::as_ptr(&ctor_type) as *mut crate::checker::types::Type;
            unsafe {
                (*t_mut).symbol = Some(Arc::clone(class_sym));
            }
        }
        ctor_type
    }

    pub(crate) fn attach_class_statics(&mut self, ctor_type: &Arc<Type>, node: &Arc<Node>) {
        let node_id = node.id();
        if self.class_statics_resolution_stack.contains(&node_id)
            || self.class_statics_resolution_stack.len() >= 200
        {
            return;
        }
        self.class_statics_resolution_stack.push(node_id);
        let mut members = SymbolTable::new();
        let mut properties: Vec<Arc<Symbol>> = Vec::new();

        if let Some(class_sym) = self.program.symbol_map().symbol_of(node) {
            let mut statics: Vec<(String, Arc<Symbol>)> = Vec::new();
            for sym in class_sym.members.entries.values() {
                if sym
                    .declarations
                    .iter()
                    .any(|d| d.has_syntactic_modifier(ModifierFlags::Static))
                {
                    statics.push((sym.name.clone(), Arc::clone(sym)));
                }
            }
            for sym in class_sym.exports.entries.values() {
                if (sym
                    .declarations
                    .iter()
                    .any(|d| d.has_syntactic_modifier(ModifierFlags::Static))
                    || sym.flags.contains(SymbolFlags::Prototype))
                    && !statics.iter().any(|(n, _)| *n == sym.name)
                {
                    statics.push((sym.name.clone(), Arc::clone(sym)));
                }
            }
            for (name, sym) in statics {
                properties.push(Arc::clone(&sym));
                members.insert(name, sym);
            }
        }

        let class_members: Option<Arc<NodeList>> = match &node.data {
            crate::ast::NodeData::ClassDeclaration(d) => Some(Arc::clone(&d.members)),
            crate::ast::NodeData::ClassExpression(d) => Some(Arc::clone(&d.members)),
            _ => None,
        };
        if let Some(member_list) = class_members {
            for member in member_list.iter() {
                if !member.has_syntactic_modifier(ModifierFlags::Static) {
                    continue;
                }
                let Some(name_node) = member.name() else {
                    continue;
                };
                let name = name_node.text().to_string();
                if name.is_empty() || members.get(&name).is_some() {
                    continue;
                }
                let flags = match member.kind {
                    SyntaxKind::MethodDeclaration => SymbolFlags::Method,
                    SyntaxKind::GetAccessor => SymbolFlags::GetAccessor,
                    SyntaxKind::SetAccessor => SymbolFlags::SetAccessor,
                    _ => SymbolFlags::Property,
                };
                let mut sym = Symbol::new(flags, name.clone());
                sym.declarations.push(Arc::clone(member));
                let sym = Arc::new(sym);

                if let crate::ast::NodeData::PropertyDeclaration(pd) = &member.data
                    && let Some(tn) = &pd.type_node
                {
                    let t = self.get_type_from_type_node(tn);
                    self.value_symbol_links.insert(
                        &sym,
                        crate::checker::types::ValueSymbolLinks {
                            resolved_type: Some(t),
                            ..Default::default()
                        },
                    );
                }
                properties.push(Arc::clone(&sym));
                members.insert(name, sym);
            }
        }

        if let Some((base_node, _)) = self.extends_base_of(node) {
            let base_ctor = self.get_type_of_class_declaration(&base_node);
            if let Some(base_structured) = base_ctor.as_structured() {
                for (name, sym) in base_structured.members.iter() {
                    if members.get(name).is_none() {
                        members.insert(name.clone(), Arc::clone(sym));
                    }
                }
                for prop in &base_structured.properties {
                    let name = prop.name.clone();
                    if members.get(&name).is_some()
                        && !properties.iter().any(|p| Arc::ptr_eq(p, prop))
                    {
                        properties.push(Arc::clone(prop));
                    }
                }
            }
        }
        self.class_statics_resolution_stack.pop();
        if members.is_empty() {
            return;
        }
        let t_mut = Arc::as_ptr(ctor_type) as *mut crate::checker::types::Type;
        unsafe {
            if let TypeData::Object(obj) = &mut (*t_mut).data {
                obj.structured.members = members;
                obj.structured.properties = properties;
            }
        }
    }

    pub(crate) fn extends_base_of(
        &self,
        class_node: &Arc<Node>,
    ) -> Option<(Arc<Node>, Arc<Symbol>)> {
        let heritage = match &class_node.data {
            crate::ast::NodeData::ClassDeclaration(data) => data.heritage_clauses.clone(),
            crate::ast::NodeData::ClassExpression(data) => data.heritage_clauses.clone(),
            _ => return None,
        };
        let extends_expr = heritage?.iter().find_map(|clause| {
            if let crate::ast::NodeData::HeritageClause(hc) = &clause.data {
                if hc.token == SyntaxKind::ExtendsKeyword {
                    return hc.types.iter().next().cloned();
                }
            }
            None
        })?;
        let base_expr = match &extends_expr.data {
            crate::ast::NodeData::ExpressionWithTypeArguments(data) => Arc::clone(&data.expression),
            _ => return None,
        };
        if base_expr.kind != SyntaxKind::Identifier {
            return None;
        }
        let symbol = self.resolve_identifier(&base_expr)?;
        if !symbol.flags.contains(SymbolFlags::Class) {
            return None;
        }
        symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ClassDeclaration)
            .cloned()
            .map(|n| (n, symbol))
    }
}
