#![allow(unused_imports)]

use crate::checker::checker_assertions_interfaces::*;

impl Checker {
    pub(crate) fn member_declared_type_for_index_check(
        &mut self,
        member: &Arc<Node>,
    ) -> Option<Arc<Type>> {
        match &member.data {
            tsox_frontend::ast::NodeData::GetAccessorDeclaration(d) => {
                Some(self.infer_function_return_type(Some(member), d.body.as_ref(), d.type_node.as_ref()))
            }
            tsox_frontend::ast::NodeData::SetAccessorDeclaration(d) => {
                let tn = d.parameters.iter().next().and_then(|p| match &p.data {
                    tsox_frontend::ast::NodeData::ParameterDeclaration(pd) => pd.type_node.clone(),
                    _ => None,
                });
                match tn {
                    Some(t) => Some(self.get_type_from_type_node(&t)),
                    None => Some(self.any_type()),
                }
            }
            tsox_frontend::ast::NodeData::PropertyDeclaration(d) => {
                if let Some(t) = &d.type_node {
                    Some(self.get_type_from_type_node(t))
                } else if let Some(init) = &d.initializer {
                    let init_t = self.get_type_of_node(init);
                    Some(self.widen_initializer_type(&init_t))
                } else {
                    None
                }
            }
            tsox_frontend::ast::NodeData::PropertySignatureDeclaration(d) => {
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

        // Go checkIndexConstraints 尾部：number 索引值型须可赋给 string 索引值型
        //（TS2413，static 侧经由构造类型同样触发）
        let string_index = index_infos.iter().find(|i| {
            i.key_type
                .as_ref()
                .is_some_and(|k| k.flags.contains(TypeFlags::String))
        });
        let number_index = index_infos.iter().find(|i| {
            i.key_type
                .as_ref()
                .is_some_and(|k| k.flags.contains(TypeFlags::Number))
        });
        if let (Some(si), Some(ni)) = (string_index, number_index)
            && let (Some(sv), Some(nv)) = (si.value_type.as_ref(), ni.value_type.as_ref())
            && !self.is_type_assignable_to(nv, sv)
        {
            // Go checkIndexConstraintForIndexSignature：错误锚点取本类型符号
            // 内声明的那个索引签名（number 优先，其次 string），都非本地且没
            // 有任何单一基类型同时持有两者时锚定接口声明名
            let parent_in_type = |d: &Arc<Node>| -> bool {
                d.parent().is_some_and(|p| {
                    t.symbol
                        .as_ref()
                        .is_some_and(|sym| sym.declarations.iter().any(|dd| Arc::ptr_eq(dd, &p)))
                })
            };
            let num_local = ni.declaration.as_ref().is_some_and(parent_in_type);
            let str_local = si.declaration.as_ref().is_some_and(parent_in_type);
            let anchor = if num_local {
                ni.declaration.as_ref().map(|d| d.loc)
            } else if str_local {
                si.declaration.as_ref().map(|d| d.loc)
            } else if declaration.kind == SyntaxKind::InterfaceDeclaration {
                let base_has_both = self.interface_base_types(declaration).iter().any(|base| {
                    let infos = self.get_index_infos_of_type(base);
                    infos.iter().any(|i| {
                        i.key_type
                            .as_ref()
                            .is_some_and(|k| k.flags.contains(TypeFlags::String))
                    }) && infos.iter().any(|i| {
                        i.key_type
                            .as_ref()
                            .is_some_and(|k| k.flags.contains(TypeFlags::Number))
                    })
                });
                if base_has_both {
                    None
                } else {
                    declaration.name().map(|n| n.loc)
                }
            } else {
                None
            };
            if let Some(name_loc) = anchor {
                let sv_str = self.type_to_string(sv);
                let nv_str = self.type_to_string(nv);
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    name_loc,
                    tsox_core::diagnostics::messages_generated::
                        X_0_INDEX_TYPE_1_IS_NOT_ASSIGNABLE_TO_2_INDEX_TYPE_3,
                    vec!["number".to_string(), nv_str, "string".to_string(), sv_str],
                ));
            }
        }

        let local_index: Option<Arc<crate::checker::IndexInfo>> = index_infos
            .iter()
            .find(|info| {
                info.declaration
                    .as_ref()
                    .and_then(|d| d.parent())
                    .is_some_and(|p| Arc::ptr_eq(&p, declaration))
            })
            .cloned();
        let is_interface = declaration.kind == SyntaxKind::InterfaceDeclaration;

        for prop in self.get_properties_of_type(t) {
            let Some(first_decl) = prop.declarations.first().cloned() else {
                continue;
            };
            if first_decl
                .parent()
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
            tsox_frontend::ast::NodeData::ClassDeclaration(d) => {
                d.members.iter().cloned().collect()
            }
            tsox_frontend::ast::NodeData::InterfaceDeclaration(d) => {
                d.members.iter().cloned().collect()
            }
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
                tsox_frontend::ast::NodeData::ClassDeclaration(cd) => cd.heritage_clauses.clone(),
                tsox_frontend::ast::NodeData::InterfaceDeclaration(id) => {
                    id.heritage_clauses.clone()
                }
                _ => continue,
            };
            let Some(clauses) = heritage else { continue };
            for clause in clauses.iter() {
                let tsox_frontend::ast::NodeData::HeritageClause(hc) = &clause.data else {
                    continue;
                };
                for type_ref in hc.types.iter() {
                    let base_expr = match &type_ref.data {
                        tsox_frontend::ast::NodeData::ExpressionWithTypeArguments(e) => {
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
                tsox_frontend::ast::NodeData::ClassDeclaration(d) => {
                    d.members.iter().cloned().collect()
                }
                tsox_frontend::ast::NodeData::InterfaceDeclaration(d) => {
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

    fn interface_base_types(&mut self, declaration: &Arc<Node>) -> Vec<Arc<Type>> {
        let tsox_frontend::ast::NodeData::InterfaceDeclaration(d) = &declaration.data else {
            return Vec::new();
        };
        let Some(clauses) = &d.heritage_clauses else {
            return Vec::new();
        };
        let mut result = Vec::new();
        for clause in clauses.iter() {
            let tsox_frontend::ast::NodeData::HeritageClause(hc) = &clause.data else {
                continue;
            };
            for h in hc.types.iter() {
                match &h.data {
                    tsox_frontend::ast::NodeData::TypeReferenceNode(_) => {
                        result.push(self.get_type_from_type_node(h));
                    }
                    tsox_frontend::ast::NodeData::ExpressionWithTypeArguments(ed) => {
                        result.push(self.get_type_of_node(&ed.expression));
                    }
                    _ => {}
                }
            }        }
        result
    }
}
