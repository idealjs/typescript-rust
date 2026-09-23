#![allow(unused_imports)]

use crate::checker::checker_imports_namespace::*;

impl Checker {
    // 裸说明符的程序内回退解析：node_modules / @types 约定路径（内存 FS 场景）
    fn resolve_bare_specifier_in_program(&self, spec: &str) -> Option<String> {
        let candidates = [
            format!("/node_modules/@types/{spec}/index.d.ts"),
            format!("/node_modules/{spec}/index.d.ts"),
            format!("/node_modules/@types/{spec}.d.ts"),
            format!("/node_modules/{spec}.d.ts"),
        ];
        for c in candidates {
            if self.program.get_source_file(&c).is_some() {
                return Some(c);
            }
        }
        None
    }

    pub(crate) fn type_of_imported_symbol(&mut self, symbol: &Arc<Symbol>) -> Option<Arc<Type>> {
        // 环守卫：`export import B = A` 的 A 又解析回 B 时无限递归；
        // Go 在 symbolLinks 里缓存解析中状态，这里以访问栈等价
        let key = symbol.id();
        if self.imported_type_resolution.contains(&key) {
            return None;
        }
        self.imported_type_resolution.push(key);
        let result = self.type_of_imported_symbol_inner(symbol);
        self.imported_type_resolution.pop();
        result
    }

    fn type_of_imported_symbol_inner(&mut self, symbol: &Arc<Symbol>) -> Option<Arc<Type>> {
        // import * as X from "m"：X 的类型是模块命名空间类型（typeof import("m")）
        if let Some(decl) = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::NamespaceImport)
        {
            let mut cur = decl.parent();
            loop {
                let Some(n) = cur else { break };
                if let tsox_frontend::ast::NodeData::ImportDeclaration(id) = &n.data {
                    let spec = id
                        .module_specifier
                        .text()
                        .trim_matches(['"', '\'', '`'])
                        .to_string();
                    let module_sym = self
                        .resolve_module_file_symbol(&spec)
                        .or_else(|| {
                            let file = self
                                .display_enclosing_file
                                .clone()
                                .or_else(|| self.current_file.clone())?;
                            let dir = match file.file_name.rfind('/') {
                                Some(i) => file.file_name[..i].to_string(),
                                None => String::new(),
                            };
                            self.resolve_module_file_symbol_in(&dir, &spec)
                        })
                        .or_else(|| {
                            let file = self
                                .display_enclosing_file
                                .clone()
                                .or_else(|| self.current_file.clone())?;
                            let path = self.program.resolve_external_module_path(
                                &spec,
                                &file.file_name,
                                tsox_core::core::compiler_options::ModuleKind::None,
                            );
                            let path = match path {
                                Some(p) => p,
                                // 内存 FS 下 @types 查找缺 package.json 时 Resolver 失败：
                                // 直接按 node_modules 约定路径匹配程序内已加载文件
                                None => self.resolve_bare_specifier_in_program(&spec)?,
                            };
                            let sf = self.program.get_source_file(&path)?;
                            self.program.symbol_map().symbol_of(&sf.node).cloned()
                        })?;
                    // 记录显示用 specifier（模块类型显示 typeof import("name")，
                    // Go 打印模块名去扩展，非原始 specifier）
                    self.module_display_specifiers.insert(
                        module_sym.id(),
                        crate::checker::nodebuilder::module_specifier_of_name(
                            &module_sym.name,
                        ),
                    );
                    return Some(self.namespace_import_module_type(&module_sym));
                }
                cur = n.parent();
            }
            return None;
        }
        if let Some(decl) = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ImportEqualsDeclaration)
        {
            let tsox_frontend::ast::NodeData::ImportEqualsDeclaration(ied) = &decl.data else {
                return None;
            };
            if ied.module_reference.kind == SyntaxKind::ExternalModuleReference {
                let ext = &ied.module_reference;
                let tsox_frontend::ast::NodeData::ExternalModuleReference(emr) = &ext.data else {
                    return None;
                };
                let module_spec = emr.expression.text().to_string();
                let module_text_trimmed = module_spec.trim_matches(['"', '\'', '`']).to_string();
                let module_sym = match self.resolve_module_file_symbol(&module_spec) {
                    Some(s) => s,
                    None => {
                        let Some(cur) = self.current_file.clone() else {
                            return None;
                        };
                        let Some(path) = self.program.resolve_external_module_path(
                            &module_text_trimmed,
                            &cur.file_name,
                            tsox_core::core::compiler_options::ModuleKind::None,
                        ) else {
                            return None;
                        };
                        let Some(sf) = self.program.get_source_file(&path) else {
                            return None;
                        };
                        let Some(sym) = self.program.symbol_map().symbol_of(&sf.node).cloned()
                        else {
                            return None;
                        };
                        sym
                    }
                };

                if let Some(eq) = module_sym
                    .exports
                    .get(tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)
                {
                    let entity_decl = eq
                        .declarations
                        .iter()
                        .find(|d| d.kind == SyntaxKind::ExportAssignment)
                        .cloned();
                    let is_entity_expression = entity_decl.as_deref().is_some_and(|d| {
                        let tsox_frontend::ast::NodeData::ExportAssignment(ea) = &d.data else {
                            return false;
                        };
                        ea.is_export_equals
                            && matches!(
                                ea.expression.kind,
                                SyntaxKind::Identifier | SyntaxKind::QualifiedName
                            )
                    });
                    if !is_entity_expression {
                        let eq = Arc::clone(eq);
                        return Some(self.get_type_of_symbol(&eq));
                    }
                    let scope_decl = module_sym
                        .declarations
                        .iter()
                        .find(|d| d.kind == SyntaxKind::ModuleDeclaration)
                        .cloned();
                    if let Some(export_decl) = entity_decl
                        && let tsox_frontend::ast::NodeData::ExportAssignment(ea) =
                            &export_decl.data
                        && ea.is_export_equals
                        && matches!(
                            ea.expression.kind,
                            SyntaxKind::Identifier | SyntaxKind::QualifiedName
                        )
                    {
                        if let Some(scope) = scope_decl {
                            self.push_scope(&scope);
                            let target = self.resolve_qualified_symbol(&ea.expression);
                            self.pop_scope();
                            if let Some(t) = target {
                                return Some(self.get_type_of_symbol(&t));
                            }
                        } else {
                            let mut segments: Vec<String> = Vec::new();
                            let mut cur = &ea.expression;
                            loop {
                                match &cur.data {
                                    tsox_frontend::ast::NodeData::Identifier(id) => {
                                        segments.push(id.text.clone());
                                        break;
                                    }
                                    tsox_frontend::ast::NodeData::QualifiedName(q) => {
                                        segments.push(q.right.text().to_string());
                                        cur = &q.left;
                                    }
                                    _ => break,
                                }
                            }
                            segments.reverse();
                            if let Some(first) = segments.first()
                                && let Some(mut target) =
                                    self.resolve_module_member_symbol(&module_sym, first, 8)
                            {
                                let mut ok = true;
                                for seg in segments.iter().skip(1) {
                                    match target
                                        .exports
                                        .get(seg)
                                        .or_else(|| target.members.get(seg))
                                        .cloned()
                                    {
                                        Some(next) => target = next,
                                        None => {
                                            ok = false;
                                            break;
                                        }
                                    }
                                }
                                if ok {
                                    return Some(self.get_type_of_symbol(&target));
                                }
                            }
                        }
                    }
                }
                return Some(self.resolve_namespace_type(&module_sym));
            }

            // import v = M.V：实体名解析到符号取声明型（Go
            // getTypeOfNode 的 QualifiedName → resolveEntityName）
            let target = &ied.module_reference;
            match &target.data {
                NodeData::Identifier(_) | NodeData::QualifiedName(_) => {
                    if let Some(sym) = self.resolve_qualified_symbol(target) {
                        return Some(self.get_type_of_symbol(&sym));
                    }
                }
                _ => {}
            }
            let t = self.get_type_of_node(target);
            if t.flags.contains(TypeFlags::Any)
                && t.intrinsic_name() == Some("any")
                && self.resolve_identifier(target).is_none()
            {
                return None;
            }
            return Some(t);
        }
        let decl = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ImportSpecifier)?;
        let name = match &decl.data {
            tsox_frontend::ast::NodeData::ImportSpecifier(d) => d
                .property_name
                .as_ref()
                .map_or_else(|| d.name.text().to_string(), |p| p.text().to_string()),
            _ => return None,
        };

        let mut import_decl = decl.parent()?;
        while !matches!(
            import_decl.data,
            tsox_frontend::ast::NodeData::ImportDeclaration(_)
        ) {
            import_decl = import_decl.parent()?;
        }
        let module_spec = match &import_decl.data {
            tsox_frontend::ast::NodeData::ImportDeclaration(d) => {
                d.module_specifier.text().to_string()
            }
            _ => return None,
        };
        let module_text_trimmed = module_spec.trim_matches(['"', '\'', '`']).to_string();
        let module_sym = match self.resolve_module_file_symbol(&module_spec) {
            Some(s) => s,
            None => {
                let Some(cur) = self.current_file.clone() else {
                    return None;
                };
                let Some(path) = self.program.resolve_external_module_path(
                    &module_text_trimmed,
                    &cur.file_name,
                    tsox_core::core::compiler_options::ModuleKind::None,
                ) else {
                    return None;
                };
                let Some(sf) = self.program.get_source_file(&path) else {
                    return None;
                };
                let Some(sym) = self.program.symbol_map().symbol_of(&sf.node).cloned() else {
                    return None;
                };
                sym
            }
        };
        let Some(member) = self.resolve_module_member_symbol(&module_sym, &name, 8) else {
            if name == "default"
                && self
                    .program
                    .options()
                    .allow_synthetic_default_imports
                    .is_true()
            {
                return Some(self.get_any_type());
            }
            return None;
        };
        if let Some(t) = self
            .value_symbol_links
            .get(&member)
            .and_then(|l| l.resolved_type.clone())
        {
            return Some(t);
        }
        for d in &member.declarations {
            match d.kind {
                SyntaxKind::FunctionDeclaration => {
                    return Some(self.get_type_of_function_like(d));
                }
                SyntaxKind::ClassDeclaration => {
                    return Some(self.get_type_of_class_declaration(d));
                }
                _ => {}
            }
        }

        Some(self.get_type_of_symbol(&member))
    }

    pub(crate) fn object_literal_export_member(
        &self,
        namespace: &Arc<Symbol>,
        name: &str,
    ) -> Option<Arc<Symbol>> {
        let ea_sym = namespace.exports.get("export=")?;
        for d in &ea_sym.declarations {
            if let tsox_frontend::ast::NodeData::ExportAssignment(ea) = &d.data
                && ea.is_export_equals
                && let tsox_frontend::ast::NodeData::ObjectLiteralExpression(ol) =
                    &ea.expression.data
            {
                for prop in ol.properties.iter() {
                    if prop.text() == name
                        && let Some(s) = self.program.symbol_map().symbol_of(prop)
                    {
                        return Some(Arc::clone(s));
                    }
                }
            }
        }
        None
    }

    pub(crate) fn heritage_type_arguments_for_base(
        &mut self,
        base_sym: &Arc<Symbol>,
    ) -> Option<Vec<Arc<Type>>> {
        let class_node = self.enclosing_class_stack.last().cloned()?;
        let heritage = match &class_node.data {
            tsox_frontend::ast::NodeData::ClassDeclaration(data) => data.heritage_clauses.clone(),
            _ => return None,
        };
        for clause in heritage?.iter() {
            let tsox_frontend::ast::NodeData::HeritageClause(hc) = &clause.data else {
                continue;
            };
            if hc.token != SyntaxKind::ExtendsKeyword {
                continue;
            }
            for type_ref in hc.types.iter() {
                let tsox_frontend::ast::NodeData::ExpressionWithTypeArguments(ewa) = &type_ref.data
                else {
                    continue;
                };
                let type_args = ewa.type_arguments.as_ref()?;
                if ewa.expression.kind == SyntaxKind::Identifier
                    && let Some(sym) = self.resolve_identifier(&ewa.expression)
                    && Arc::ptr_eq(&sym, base_sym)
                {
                    return Some(
                        type_args
                            .iter()
                            .map(|t| self.get_type_from_type_node(t))
                            .collect(),
                    );
                }
            }
        }
        None
    }
}
