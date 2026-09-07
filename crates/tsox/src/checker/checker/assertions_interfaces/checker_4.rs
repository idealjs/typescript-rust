#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn member_declared_type_for_index_check(
        &mut self,
        member: &Arc<Node>,
    ) -> Option<Arc<Type>> {
        match &member.data {
            crate::ast::NodeData::GetAccessorDeclaration(d) => {
                Some(self.infer_function_return_type(d.body.as_ref(), d.type_node.as_ref()))
            }
            crate::ast::NodeData::SetAccessorDeclaration(d) => {
                let tn = d.parameters.iter().next().and_then(|p| match &p.data {
                    crate::ast::NodeData::ParameterDeclaration(pd) => pd.type_node.clone(),
                    _ => None,
                });
                match tn {
                    Some(t) => Some(self.get_type_from_type_node(&t)),
                    None => Some(self.any_type()),
                }
            }
            crate::ast::NodeData::PropertyDeclaration(d) => {
                if let Some(t) = &d.type_node {
                    Some(self.get_type_from_type_node(t))
                } else if let Some(init) = &d.initializer {
                    let init_t = self.get_type_of_node(init);
                    Some(self.widen_initializer_type(&init_t))
                } else {
                    None
                }
            }
            crate::ast::NodeData::PropertySignatureDeclaration(d) => {
                Some(self.get_type_from_type_node(&d.type_node))
            }
            _ => None,
        }
    }

    pub(crate) fn check_index_constraints(&mut self, t: &Arc<Type>, declaration: &Arc<Node>) {
        let index_infos = self.get_index_infos_of_type(t);
        if index_infos.is_empty() {
            return;
        }

        let local_index: Option<Arc<crate::checker::IndexInfo>> = index_infos
            .iter()
            .find(|info| {
                info.declaration
                    .as_ref()
                    .and_then(|d| d.parent.as_ref())
                    .is_some_and(|p| Arc::ptr_eq(p, declaration))
            })
            .cloned();
        let is_interface = declaration.kind == SyntaxKind::InterfaceDeclaration;

        for prop in self.get_properties_of_type(t) {
            let Some(first_decl) = prop.declarations.first().cloned() else {
                continue;
            };
            if first_decl
                .parent
                .as_ref()
                .is_some_and(|p| Arc::ptr_eq(p, declaration))
            {
                continue;
            }
            let Some(name) = Self::member_name_node(&first_decl) else {
                continue;
            };
            if name.kind == SyntaxKind::ComputedPropertyName {
                continue;
            }
            let Some(key_type) = self.property_name_key_type(&name) else {
                continue;
            };
            let prop_type = self.get_type_of_symbol(&prop);
            let display = self.property_name_display(&name);
            self.check_index_constraint_for_property(
                t,
                &key_type,
                &prop_type,
                &name,
                &display,
                None,
                local_index.clone(),
                is_interface.then(|| Arc::clone(declaration)),
                &index_infos,
            );
        }

        let props_by_name: std::collections::HashMap<String, Arc<Symbol>> = self
            .get_properties_of_type(t)
            .into_iter()
            .map(|p| (p.name.clone(), p))
            .collect();
        let members: Vec<Arc<Node>> = match &declaration.data {
            crate::ast::NodeData::ClassDeclaration(d) => d.members.iter().cloned().collect(),
            crate::ast::NodeData::InterfaceDeclaration(d) => d.members.iter().cloned().collect(),
            _ => Vec::new(),
        };
        for member in &members {
            if member.kind == SyntaxKind::IndexSignature {
                continue;
            }
            let Some(name) = Self::member_name_node(member) else {
                continue;
            };
            let member_symbol = self.program.symbol_map().symbol_of(member).cloned();
            let Some(key_type) = self.property_name_key_type(&name) else {
                continue;
            };
            let prop_type = if name.kind != SyntaxKind::ComputedPropertyName {
                match props_by_name.get(name.text()) {
                    Some(sym) => self.get_type_of_symbol(sym),
                    None => match self.member_declared_type_for_index_check(member) {
                        Some(t) => t,
                        None => continue,
                    },
                }
            } else {
                match self.member_declared_type_for_index_check(member) {
                    Some(t) => t,
                    None => match &member_symbol {
                        Some(sym) => self.get_type_of_symbol(sym),
                        None => continue,
                    },
                }
            };
            let display = self.property_name_display(&name);
            let local_name_node = Some(Arc::clone(&name));
            self.check_index_constraint_for_property(
                t,
                &key_type,
                &prop_type,
                &name,
                &display,
                local_name_node,
                local_index.clone(),
                is_interface.then(|| Arc::clone(declaration)),
                &index_infos,
            );
        }

        let mut bases: Vec<Arc<Node>> = Vec::new();
        let mut worklist: Vec<Arc<Node>> = vec![Arc::clone(declaration)];
        let mut guard = 0;
        while let Some(d) = worklist.pop() {
            guard += 1;
            if guard > 32 {
                break;
            }
            let heritage = match &d.data {
                crate::ast::NodeData::ClassDeclaration(cd) => cd.heritage_clauses.clone(),
                crate::ast::NodeData::InterfaceDeclaration(id) => id.heritage_clauses.clone(),
                _ => continue,
            };
            let Some(clauses) = heritage else { continue };
            for clause in clauses.iter() {
                let crate::ast::NodeData::HeritageClause(hc) = &clause.data else {
                    continue;
                };
                for type_ref in hc.types.iter() {
                    let base_expr = match &type_ref.data {
                        crate::ast::NodeData::ExpressionWithTypeArguments(e) => {
                            Arc::clone(&e.expression)
                        }
                        _ => continue,
                    };
                    let base_symbol = if base_expr.kind == SyntaxKind::Identifier {
                        self.resolve_identifier(&base_expr)
                    } else {
                        None
                    };
                    let Some(base_symbol) = base_symbol else {
                        continue;
                    };
                    for bd in &base_symbol.declarations {
                        if matches!(
                            bd.kind,
                            SyntaxKind::ClassDeclaration | SyntaxKind::InterfaceDeclaration
                        ) && !bases.iter().any(|b| Arc::ptr_eq(b, bd))
                            && !Arc::ptr_eq(bd, &d)
                        {
                            bases.push(Arc::clone(bd));
                            worklist.push(Arc::clone(bd));
                        }
                    }
                }
            }
        }
        for base in &bases {
            let base_members: Vec<Arc<Node>> = match &base.data {
                crate::ast::NodeData::ClassDeclaration(d) => d.members.iter().cloned().collect(),
                crate::ast::NodeData::InterfaceDeclaration(d) => {
                    d.members.iter().cloned().collect()
                }
                _ => continue,
            };
            for member in base_members {
                let Some(name) = Self::member_name_node(&member) else {
                    continue;
                };
                if name.kind != SyntaxKind::ComputedPropertyName {
                    continue;
                }
                let Some(key_type) = self.property_name_key_type(&name) else {
                    continue;
                };
                let Some(symbol) = self.program.symbol_map().symbol_of(&member).cloned() else {
                    continue;
                };
                let prop_type = self
                    .member_declared_type_for_index_check(&member)
                    .unwrap_or_else(|| self.get_type_of_symbol(&symbol));
                let display = self.property_name_display(&name);
                let index_for_error = local_index.clone();
                let iface_decl = is_interface.then(|| Arc::clone(declaration));
                self.check_index_constraint_for_property(
                    t,
                    &key_type,
                    &prop_type,
                    &name,
                    &display,
                    None,
                    index_for_error,
                    iface_decl,
                    &index_infos,
                );
            }
        }
    }
}
