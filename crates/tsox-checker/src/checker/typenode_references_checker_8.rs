#![allow(unused_imports)]

use crate::checker::typenode_references::*;

impl Checker {
    pub(crate) fn resolve_namespace_type(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        // namespace+interface 合并符号：接口声明类型（type_alias_links）与
        // 模块实例类型（值侧）分离缓存，避免互相污染
        let merged_with_type_meaning = symbol.flags.contains(SymbolFlags::Interface);
        if merged_with_type_meaning {
            if let Some(cached) = self.merged_ns_instance_type_cache.get(&symbol.id()) {
                return Arc::clone(cached);
            }
        } else if let Some(cached) = self
            .type_alias_links
            .get(symbol)
            .and_then(|l| l.declared_type.clone())
        {
            return cached;
        }
        if !self.push_type_resolution(
            Arc::as_ptr(symbol) as *const Symbol,
            crate::checker::TypeResolutionProperty::Type,
        ) {
            return self.error_type();
        }
        let result = self.resolve_namespace_type_uncached(symbol);
        self.pop_type_resolution();
        result
    }

    fn resolve_namespace_type_uncached(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        let mut members: Vec<(String, Arc<Symbol>)> = symbol
            .exports
            .iter()
            .filter(|(_, v)| {
                // type-only 导出别名不属于模块实例的值成员
                !(v.flags.contains(SymbolFlags::Alias)
                    && !v.flags.intersects(SymbolFlags::VALUE.union(SymbolFlags::Class))
                    && self.is_type_only_alias_declaration(v))
            })
            .map(|(k, v)| (k.clone(), Arc::clone(v)))
            .collect();

        let is_global_this = self
            .global_this_symbol
            .as_ref()
            .is_some_and(|g| Arc::ptr_eq(g, symbol));
        if is_global_this {
            for (name, sym) in self.globals.iter() {
                if !members.iter().any(|(n, _)| n == name) {
                    members.push((name.clone(), Arc::clone(sym)));
                }
            }
        }

        // Go binder 静态成员入 exports；本仓静态在 members：clodule 合并类型
        // 须并入类静态成员（typeof A 含 bar/x/baz 的成员来源）
        if symbol.flags.contains(SymbolFlags::Class) {
            for sym in symbol.members.entries.values() {
                if sym
                    .declarations
                    .iter()
                    .any(|d| {
                        d.has_syntactic_modifier(tsox_frontend::ast::ModifierFlags::Static)
                    })
                    && !members.iter().any(|(n, _)| *n == sym.name)
                {
                    members.push((sym.name.clone(), Arc::clone(sym)));
                }
            }
        }

        if self.ambient_namespace_locals_visible(symbol) {
            let local_members: Vec<(String, Arc<Symbol>)> = symbol
                .declarations
                .iter()
                .filter(|d| d.kind == SyntaxKind::ModuleDeclaration)
                .filter_map(|d| {
                    self.program.symbol_map().locals.get(&d.id()).map(|l| {
                        l.iter()
                            .map(|(k, v)| (k.clone(), Arc::clone(v)))
                            .collect::<Vec<(String, Arc<Symbol>)>>()
                    })
                })
                .flatten()
                .collect();
            for (k, v) in local_members {
                if !members.iter().any(|(mk, _)| *mk == k) {
                    members.push((k, v));
                }
            }
        }

        let mut file_exported: Vec<(String, Arc<Symbol>)> = Vec::new();
        if symbol
            .declarations
            .iter()
            .any(|d| d.kind == SyntaxKind::SourceFile)
            && members.is_empty()
        {
            let sym_map = self.program.symbol_map();

            let mut wanted: Vec<(String, Option<String>)> = Vec::new();
            let mut default_node: Option<Arc<Node>> = None;
            self.for_each_module_statement(symbol, |stmt| {
                let has_export =
                    stmt.has_syntactic_modifier(tsox_frontend::ast::ModifierFlags::Export);
                match &stmt.data {
                    NodeData::ExportDeclaration(d) => {
                        if let Some(clause) = &d.export_clause
                            && let NodeData::NamedExports(ne) = &clause.data
                        {
                            for el in ne.elements.iter() {
                                if let NodeData::ExportSpecifier(spec) = &el.data
                                    // type-only 导出不属于模块实例的值成员
                                    && !(d.is_type_only || spec.is_type_only)
                                {
                                    let exported =
                                        spec.name.text().trim_matches(['"', '\'', '`']).to_string();
                                    let local = spec
                                        .property_name
                                        .as_ref()
                                        .unwrap_or(&spec.name)
                                        .text()
                                        .trim_matches(['"', '\'', '`'])
                                        .to_string();
                                    wanted.push((exported, Some(local)));
                                }
                            }
                        }
                    }
                    NodeData::ExportAssignment(ea) => {
                        if !ea.is_export_equals && default_node.is_none() {
                            default_node = Some(Arc::clone(stmt));
                        }
                    }
                    NodeData::VariableStatement(vs) if has_export => {
                        if let NodeData::VariableDeclarationList(vdl) = &vs.declaration_list.data {
                            for decl in vdl.declarations.iter() {
                                if let Some(name) = decl.name() {
                                    wanted.push((name.text().to_string(), None));
                                }
                            }
                        }
                    }
                    _ if has_export => {
                        if let Some(name) = stmt.name() {
                            wanted.push((name.text().to_string(), None));
                        }
                    }
                    _ => {}
                }
                false
            });
            let file_node = symbol
                .declarations
                .iter()
                .find(|d| d.kind == SyntaxKind::SourceFile);
            let locals = file_node.and_then(|f| sym_map.locals.get(&f.id()));
            for (exported, clause_local) in wanted.iter() {
                if members.iter().any(|(k, _)| *k == *exported)
                    || file_exported.iter().any(|(k, _)| *k == *exported)
                {
                    continue;
                }
                let lookup = clause_local.as_deref().unwrap_or(exported);
                if let Some(s) = locals.and_then(|l| l.get(lookup).cloned()) {
                    file_exported.push((exported.clone(), s));
                } else if let Some(s) = symbol.members.get(lookup).cloned() {
                    file_exported.push((exported.clone(), s));
                }
            }
            if let Some(node) = default_node
                && !members.iter().any(|(k, _)| k == "default")
                && !file_exported.iter().any(|(k, _)| k == "default")
            {
                let s = sym_map.symbol_of(&node).cloned().or_else(|| {
                    node.expression()
                        .and_then(|e| sym_map.symbol_of(e).cloned())
                });
                if let Some(s) = s {
                    file_exported.push(("default".to_string(), s));
                }
            }
        }

        let mut reexported: Vec<(String, Arc<Symbol>)> = Vec::new();
        {
            let mut clause_specs: Vec<(String, String, String)> = Vec::new();
            self.for_each_module_statement(symbol, |stmt| {
                if let NodeData::ExportDeclaration(d) = &stmt.data
                    && let Some(clause) = &d.export_clause
                    && let NodeData::NamedExports(ne) = &clause.data
                    && let Some(module_spec) = &d.module_specifier
                {
                    for el in ne.elements.iter() {
                        if let NodeData::ExportSpecifier(spec) = &el.data
                            && !(d.is_type_only || spec.is_type_only)
                        {
                            let exported =
                                spec.name.text().trim_matches(['"', '\'', '`']).to_string();
                            let imported = spec
                                .property_name
                                .as_ref()
                                .unwrap_or(&spec.name)
                                .text()
                                .trim_matches(['"', '\'', '`'])
                                .to_string();
                            let module_text = module_spec
                                .text()
                                .trim_matches(['"', '\'', '`'])
                                .to_string();
                            if !exported.is_empty() && !module_text.is_empty() {
                                clause_specs.push((exported, imported, module_text));
                            }
                        }
                    }
                }
                false
            });
            for (exported, imported, module_text) in clause_specs {
                if members.iter().any(|(k, _)| *k == exported)
                    || reexported.iter().any(|(k, _)| *k == exported)
                {
                    continue;
                }
                let target = self
                    .resolve_module_spec_from(symbol, &module_text)
                    .and_then(|m| self.resolve_module_member_symbol(&m, &imported, 8));
                if let Some(t) = target {
                    reexported.push((exported, t));
                }
            }
        }
        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();
        for (name, member_sym) in members
            .iter()
            .chain(file_exported.iter())
            .chain(reexported.iter())
        {
            if name.starts_with(tsox_frontend::ast::INTERNAL_SYMBOL_NAME_PREFIX)
                || name == tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS
            {
                continue;
            }

            // 保留成员原始身份（flags/声明/父链），显示 var A.Y 等限定前缀用
            let prop_sym = if is_global_this {
                Arc::clone(member_sym)
            } else {
                let s = Arc::as_ptr(member_sym) as *mut Symbol;
                unsafe {
                    if (*s).parent().is_none() {
                        (*s).set_parent(symbol);
                    }
                }
                Arc::clone(member_sym)
            };
            symbol_table.insert(name.clone(), Arc::clone(&prop_sym));
            // Go setStructuredTypeMembers→getNamedMembers→symbolIsValue：
            // properties 列表只收值成员；别名揭示落点无值义（Go 侧回落
            // unknownSymbol，Property|Variable 属 Value）时按值成员计
            let is_value = member_sym.flags.intersects(SymbolFlags::VALUE)
                || (member_sym.flags.contains(SymbolFlags::Alias) && {
                    let base = self.resolve_alias_base(Arc::clone(member_sym));
                    base.flags.intersects(SymbolFlags::VALUE)
                        || base.flags == SymbolFlags::Alias
                        || base.flags == SymbolFlags::Alias.union(SymbolFlags::Assignment)
                });
            if is_value {
                props.push(prop_sym);
            }
        }
        let result = Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: crate::checker::types::next_type_id(),
            symbol: Some(Arc::clone(symbol)),
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: symbol_table,
                    properties: props,
                    ..Default::default()
                },
                ..Default::default()
            }),
        });
        let merged_with_type_meaning = symbol.flags.contains(SymbolFlags::Interface);
        if merged_with_type_meaning {
            self.merged_ns_instance_type_cache
                .insert(symbol.id(), Arc::clone(&result));
        } else {
            self.type_alias_links.get_or_default(symbol).declared_type = Some(Arc::clone(&result));
        }
        result
    }
}
