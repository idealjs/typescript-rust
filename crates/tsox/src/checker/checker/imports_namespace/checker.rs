#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn enclosing_function_is_generator(&self, node: &Arc<Node>) -> bool {
        let mut cur = node.parent.clone();
        while let Some(n) = cur {
            let in_name_of_current =
                crate::ast::node_data_generated::node_name(&n).is_some_and(|name| {
                    name.loc.pos() <= node.loc.pos() && node.loc.end() <= name.loc.end()
                });
            if in_name_of_current {
                cur = n.parent.clone();
                continue;
            }
            match &n.data {
                crate::ast::NodeData::FunctionDeclaration(d) => {
                    return d.asterisk_token.is_some();
                }
                crate::ast::NodeData::FunctionExpression(d) => {
                    return d.asterisk_token.is_some();
                }
                crate::ast::NodeData::MethodDeclaration(d) => {
                    return d.asterisk_token.is_some();
                }

                crate::ast::NodeData::ArrowFunction(_)
                | crate::ast::NodeData::GetAccessorDeclaration(_)
                | crate::ast::NodeData::SetAccessorDeclaration(_)
                | crate::ast::NodeData::ConstructorDeclaration(_) => return false,
                _ => {}
            }
            cur = n.parent.clone();
        }
        false
    }

    pub(crate) fn get_array_element_type(&self, t: &Arc<Type>) -> Arc<Type> {
        match &t.data {
            crate::checker::TypeData::Object(obj) => {
                if let Some(elem) = obj.type_arguments.first() {
                    return Arc::clone(elem);
                }
                self.get_any_type()
            }
            crate::checker::TypeData::EvolvingArray(ea) => ea
                .element_type
                .clone()
                .unwrap_or_else(|| self.get_any_type()),
            _ => self.get_any_type(),
        }
    }

    pub(crate) fn is_empty_array_literal(&self, node: &Arc<Node>) -> bool {
        matches!(
            &node.data,
            crate::ast::NodeData::ArrayLiteralExpression(d) if d.elements.is_empty()
        )
    }

    pub(crate) fn get_missing_required_properties(
        &self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> Vec<String> {
        let Some(source_struct) = source.as_structured() else {
            return Vec::new();
        };
        let Some(target_struct) = target.as_structured() else {
            return Vec::new();
        };
        let mut missing = Vec::new();
        for target_prop in &target_struct.properties {
            if target_prop.flags.contains(SymbolFlags::Optional) {
                continue;
            }
            if source_struct.members.get(&target_prop.name).is_none() {
                missing.push(target_prop.name.clone());
            }
        }
        missing
    }

    pub(crate) fn get_property_name_from_node(&self, node: &Arc<Node>) -> String {
        match &node.data {
            NodeData::Identifier(id) => id.text.clone(),
            NodeData::StringLiteral(s) => s.text.clone(),
            NodeData::NumericLiteral(n) => n.text.clone(),
            NodeData::ComputedPropertyName(_) => {
                let file = self
                    .get_source_file_of_node(node)
                    .or_else(|| self.current_file.clone());
                let Some(file) = file else {
                    return String::new();
                };
                let pos = node.loc.pos();
                let end = node.loc.end();
                if pos < end && end <= file.text.len() {
                    file.text[pos..end].to_string()
                } else {
                    String::new()
                }
            }
            _ => node.text().to_string(),
        }
    }

    pub(crate) fn build_class_instance_type_with_base(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (members, heritage_clauses) = match &node.data {
            crate::ast::NodeData::ClassDeclaration(data) => {
                (&data.members, data.heritage_clauses.clone())
            }

            crate::ast::NodeData::ClassExpression(data) => {
                (&data.members, data.heritage_clauses.clone())
            }
            _ => return self.build_interface_type_from_members(&Arc::new(NodeList::default())),
        };

        let own_type = self.build_interface_type_from_members(members);

        if let Some(class_sym) = self.program.symbol_map().symbol_of(node) {
            let own_mut = Arc::as_ptr(&own_type) as *mut crate::checker::types::Type;
            unsafe {
                (*own_mut).symbol = Some(Arc::clone(class_sym));
            }
        }

        let mut base_type: Option<Arc<Type>> = None;
        if let Some(ref heritage) = heritage_clauses {
            for clause in heritage.iter() {
                if let crate::ast::NodeData::HeritageClause(hc) = &clause.data {
                    if hc.token == SyntaxKind::ExtendsKeyword {
                        if let Some(type_ref) = hc.types.iter().next() {
                            base_type = Some(self.resolve_base_class_instance_type(type_ref));
                        }
                        break;
                    }
                }
            }
        }
        match base_type {
            Some(base) => self.merge_instance_types(&own_type, &base),
            None => own_type,
        }
    }

    pub(crate) fn get_constituent_property(
        &mut self,
        object_type: &Arc<Type>,
        name: &str,
    ) -> Option<std::sync::Arc<crate::ast::Symbol>> {
        let apparent = self.get_apparent_type(object_type);
        let parts: Vec<Arc<Type>> = if apparent
            .flags
            .contains(crate::checker::types::TypeFlags::Union)
        {
            match &apparent.data {
                crate::checker::types::TypeData::Union(u) => u.union_or_intersection.types.clone(),
                _ => vec![apparent],
            }
        } else {
            vec![apparent]
        };
        for p in parts {
            if let Some(sym) = self.get_property_of_type(&p, name) {
                return Some(sym);
            }
        }
        None
    }

    pub(crate) fn loop_has_escaping_break(n: &Arc<Node>, direct: bool) -> bool {
        match n.kind {
            SyntaxKind::BreakStatement => {
                matches!(
                    &n.data,
                    crate::ast::NodeData::BreakStatement(d) if d.label.is_some()
                ) || direct
            }
            SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::Constructor => false,
            _ => {
                let nested = matches!(
                    n.kind,
                    SyntaxKind::WhileStatement
                        | SyntaxKind::DoStatement
                        | SyntaxKind::ForStatement
                        | SyntaxKind::ForInStatement
                        | SyntaxKind::ForOfStatement
                        | SyntaxKind::SwitchStatement
                );
                let mut found = false;
                crate::ast::node_data_generated::for_each_child(n, |child| {
                    if Self::loop_has_escaping_break(child, direct && !nested) {
                        found = true;
                        true
                    } else {
                        false
                    }
                });
                found
            }
        }
    }

    pub(crate) fn function_body_has_explicit_return(body: &Arc<Node>) -> bool {
        fn walk(n: &Arc<Node>) -> bool {
            match n.kind {
                SyntaxKind::ReturnStatement => return true,

                SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionExpression
                | SyntaxKind::ArrowFunction
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor => return false,
                _ => {}
            }
            let mut found = false;
            crate::ast::node_data_generated::for_each_child(n, |child| {
                if walk(child) {
                    found = true;
                    true
                } else {
                    false
                }
            });
            found
        }
        walk(body)
    }

    pub(crate) fn has_same_named_type_symbol(&self, name: &str) -> bool {
        let type_meaning = SymbolFlags::Interface
            | SymbolFlags::Class
            | SymbolFlags::TypeParameter
            | SymbolFlags::TypeAlias
            | SymbolFlags::RegularEnum
            | SymbolFlags::ConstEnum;
        let symbol_map = self.program.symbol_map();
        for &container_id in self.scope_stack.iter().rev() {
            if let Some(locals) = symbol_map.locals.get(&container_id)
                && let Some(sym) = locals.get(name)
                && sym.flags.intersects(type_meaning)
            {
                return true;
            }
            if let Some(container_sym) = symbol_map.symbols.get(&container_id)
                && (container_sym
                    .members
                    .get(name)
                    .is_some_and(|s| s.flags.intersects(type_meaning))
                    || container_sym
                        .exports
                        .get(name)
                        .is_some_and(|s| s.flags.intersects(type_meaning)))
            {
                return true;
            }
        }
        self.globals
            .get(name)
            .is_some_and(|s| s.flags.intersects(type_meaning))
    }

    pub(crate) fn namespace_usable_as_value(&mut self, namespace: &Arc<Symbol>) -> bool {
        let state_instantiated = namespace
            .declarations
            .iter()
            .filter(|d| d.kind == SyntaxKind::ModuleDeclaration)
            .any(|d| {
                module_is_instantiated(d, self.compiler_options.should_preserve_const_enums())
            });
        state_instantiated || self.namespace_has_value_side(namespace)
    }
}
