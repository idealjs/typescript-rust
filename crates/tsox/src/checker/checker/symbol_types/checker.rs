#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn get_type_of_symbol(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        if symbol.flags.contains(SymbolFlags::Alias) {
            let target = self.follow_alias(symbol);
            if let Some(target) = target
                && !Arc::ptr_eq(&target, symbol)
            {
                let t = self.get_type_of_symbol(&target);
                self.value_symbol_links.get_or_default(symbol).resolved_type = Some(Arc::clone(&t));
                return t;
            }
            return self.get_any_type();
        }

        if symbol.flags.contains(SymbolFlags::ValueModule)
            && (symbol.flags.contains(SymbolFlags::Function)
                || symbol.flags.contains(SymbolFlags::Class)
                || symbol.flags.contains(SymbolFlags::RegularEnum)
                || symbol.flags.contains(SymbolFlags::ConstEnum))
        {
            return self.get_type_of_merged_namespace_symbol(symbol);
        }

        if symbol.flags.contains(SymbolFlags::Prototype) {
            if let Some(links) = self.value_symbol_links.get(symbol) {
                if let Some(ref t) = links.resolved_type {
                    return Arc::clone(t);
                }
            }
            let result = self.get_type_of_prototype_property(symbol);
            self.value_symbol_links.get_or_default(symbol).resolved_type = Some(result.clone());
            return result;
        }

        if symbol.flags.intersects(SymbolFlags::Method)
            && let Some(decl) = symbol
                .declarations
                .iter()
                .find(|d| d.kind == SyntaxKind::MethodDeclaration)
            && let crate::ast::NodeData::MethodDeclaration(data) = &decl.data
        {
            if let Some(links) = self.value_symbol_links.get(symbol)
                && let Some(ref t) = links.resolved_type
            {
                return Arc::clone(t);
            }
            self.push_scope(decl);
            let return_type = match data.type_node.as_ref() {
                Some(tn) => self.get_type_from_type_node(tn),
                None => self.get_any_type(),
            };
            let sig = self.build_signature_from_function_like_type_node(
                &data.parameters,
                return_type,
                false,
                None,
                Some(Arc::clone(decl)),
            );
            self.pop_scope();
            let t = self.create_function_or_constructor_type(vec![sig], false);
            self.value_symbol_links.get_or_default(symbol).resolved_type = Some(Arc::clone(&t));
            return t;
        }

        if symbol.flags.contains(SymbolFlags::BlockScopedVariable)
            || symbol.flags.contains(SymbolFlags::FunctionScopedVariable)
            || symbol.flags.contains(SymbolFlags::Function)
            || symbol.flags.contains(SymbolFlags::Class)
            || symbol.flags.contains(SymbolFlags::Property)
            || symbol.flags.contains(SymbolFlags::EnumMember)
        {
            if let Some(links) = self.value_symbol_links.get(symbol) {
                if let Some(ref t) = links.resolved_type {
                    return Arc::clone(t);
                }
            }

            if let Some(decl) = &symbol.value_declaration {
                if let Some(links) = self.type_node_links.get(decl) {
                    if let Some(ref t) = links.resolved_type {
                        return Arc::clone(t);
                    }
                }
            }

            for decl in &symbol.declarations {
                if let Some(links) = self.type_node_links.get(decl) {
                    if let Some(ref t) = links.resolved_type {
                        return Arc::clone(t);
                    }
                }
            }

            if let Some(t) = self.resolve_symbol_declared_type_on_demand(symbol) {
                self.value_symbol_links.get_or_default(symbol).resolved_type = Some(Arc::clone(&t));
                return t;
            }
            self.get_any_type()
        } else if symbol.flags.contains(SymbolFlags::ValueModule) {
            self.resolve_namespace_type(symbol)
        } else if symbol.flags.intersects(SymbolFlags::ENUM) {
            self.resolve_enum_value_type(symbol)
        } else {
            self.get_any_type()
        }
    }

    pub(crate) fn attach_function_expando_type(
        &mut self,
        symbol: &Arc<crate::ast::Symbol>,
        base: Arc<Type>,
    ) -> Arc<Type> {
        let mut entries: Vec<(String, Arc<Node>)> = Vec::new();
        for (name, sym) in symbol.exports.iter() {
            if name == crate::ast::INTERNAL_SYMBOL_NAME_ASSIGNMENT {
                for d in &sym.declarations {
                    let mname = match &d.data {
                        crate::ast::NodeData::BinaryExpression(b) => match &b.left.data {
                            crate::ast::NodeData::ElementAccessExpression(eae) => self
                                .node_source_text(&eae.argument_expression)
                                .map(|t| format!("[{t}]"))
                                .unwrap_or_default(),
                            _ => String::new(),
                        },
                        _ => String::new(),
                    };
                    entries.push((mname, Arc::clone(d)));
                }
            } else if sym.flags.contains(SymbolFlags::Property)
                && !sym.declarations.is_empty()
                && sym
                    .declarations
                    .iter()
                    .all(|d| d.kind == SyntaxKind::BinaryExpression)
            {
                for d in &sym.declarations {
                    entries.push((name.clone(), Arc::clone(d)));
                }
            }
        }
        if entries.is_empty() {
            return base;
        }
        let mut table = crate::ast::SymbolTable::new();
        let mut props: Vec<Arc<crate::ast::Symbol>> = Vec::new();
        for (name, node) in entries {
            if table.entries.contains_key(&name) {
                continue;
            }
            let crate::ast::NodeData::BinaryExpression(bin) = &node.data else {
                continue;
            };
            let rhs_type = self.with_declaring_file_context(&node, |c| {
                let t = c.get_type_of_node(&bin.right);
                c.get_widened_type(&t)
            });
            let prop = Arc::new(crate::ast::Symbol::new(SymbolFlags::Property, name.clone()));
            self.value_symbol_links.insert(
                &prop,
                ValueSymbolLinks {
                    resolved_type: Some(rhs_type),
                    ..Default::default()
                },
            );
            table.insert(name.clone(), Arc::clone(&prop));
            props.push(prop);
        }
        if props.is_empty() {
            return base;
        }
        let face = Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: crate::checker::types::next_type_id(),
            symbol: Some(Arc::clone(symbol)),
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: table,
                    properties: props,
                    ..Default::default()
                },
                ..Default::default()
            }),
        });
        Arc::new(Type {
            flags: TypeFlags::Intersection,
            object_flags: ObjectFlags::None,
            id: crate::checker::types::next_type_id(),
            symbol: None,
            alias: None,
            data: TypeData::Intersection(IntersectionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: vec![base, face],
                },
                ..Default::default()
            }),
        })
    }

    pub(crate) fn add_optional_undefined(&mut self, t: Arc<Type>) -> Arc<Type> {
        if !self.strict_null_checks {
            return t;
        }

        if t.flags.contains(TypeFlags::Any) && t.intrinsic_name() == Some("error") {
            return t;
        }
        let already = t.flags.contains(TypeFlags::Undefined)
            || (t.flags.contains(TypeFlags::Union)
                && t.types()
                    .is_some_and(|ts| ts.iter().any(|c| c.flags.contains(TypeFlags::Undefined))));
        if already {
            return t;
        }
        self.get_union_type(vec![t, self.undefined_type()])
    }

    pub(crate) fn strip_optional_undefined(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.contains(TypeFlags::Union)
            && let Some(ts) = t.types()
        {
            let kept: Vec<Arc<Type>> = ts
                .iter()
                .filter(|c| !c.flags.contains(TypeFlags::Undefined))
                .cloned()
                .collect();
            if !kept.is_empty() && kept.len() != ts.len() {
                return if kept.len() == 1 {
                    kept.into_iter().next().expect("nonempty")
                } else {
                    self.get_union_type(kept)
                };
            }
        }
        Arc::clone(t)
    }
}
