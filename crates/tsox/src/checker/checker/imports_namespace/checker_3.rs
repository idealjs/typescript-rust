#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn type_of_imported_symbol(&mut self, symbol: &Arc<Symbol>) -> Option<Arc<Type>> {
        if let Some(decl) = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ImportEqualsDeclaration)
        {
            let crate::ast::NodeData::ImportEqualsDeclaration(ied) = &decl.data else {
                return None;
            };
            if ied.module_reference.kind == SyntaxKind::ExternalModuleReference {
                let ext = &ied.module_reference;
                let crate::ast::NodeData::ExternalModuleReference(emr) = &ext.data else {
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
                            crate::core::compiler_options::ModuleKind::None,
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
                    .get(crate::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)
                {
                    let entity_decl = eq
                        .declarations
                        .iter()
                        .find(|d| d.kind == SyntaxKind::ExportAssignment)
                        .cloned();
                    let scope_decl = module_sym
                        .declarations
                        .iter()
                        .find(|d| d.kind == SyntaxKind::ModuleDeclaration)
                        .cloned();
                    if let Some(export_decl) = entity_decl
                        && let crate::ast::NodeData::ExportAssignment(ea) = &export_decl.data
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
                                    crate::ast::NodeData::Identifier(id) => {
                                        segments.push(id.text.clone());
                                        break;
                                    }
                                    crate::ast::NodeData::QualifiedName(q) => {
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

            let target = &ied.module_reference;
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
            crate::ast::NodeData::ImportSpecifier(d) => d
                .property_name
                .as_ref()
                .map_or_else(|| d.name.text().to_string(), |p| p.text().to_string()),
            _ => return None,
        };

        let mut import_decl = decl.parent.as_ref()?;
        while !matches!(import_decl.data, crate::ast::NodeData::ImportDeclaration(_)) {
            import_decl = import_decl.parent.as_ref()?;
        }
        let module_spec = match &import_decl.data {
            crate::ast::NodeData::ImportDeclaration(d) => d.module_specifier.text().to_string(),
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
                    crate::core::compiler_options::ModuleKind::None,
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
            if let crate::ast::NodeData::ExportAssignment(ea) = &d.data
                && ea.is_export_equals
                && let crate::ast::NodeData::ObjectLiteralExpression(ol) = &ea.expression.data
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
            crate::ast::NodeData::ClassDeclaration(data) => data.heritage_clauses.clone(),
            _ => return None,
        };
        for clause in heritage?.iter() {
            let crate::ast::NodeData::HeritageClause(hc) = &clause.data else {
                continue;
            };
            if hc.token != SyntaxKind::ExtendsKeyword {
                continue;
            }
            for type_ref in hc.types.iter() {
                let crate::ast::NodeData::ExpressionWithTypeArguments(ewa) = &type_ref.data else {
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
