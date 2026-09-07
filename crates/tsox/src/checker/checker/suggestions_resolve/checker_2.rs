#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn collect_unimplemented_abstract_members(
        class: &Arc<Node>,
        base: &Arc<Node>,
        out: &mut Vec<String>,
    ) {
        for member in Self::class_members_of(base).iter() {
            let (name_node, is_abstract_member) = match &member.data {
                crate::ast::NodeData::PropertyDeclaration(d) => (
                    &d.name,
                    member.has_syntactic_modifier(ModifierFlags::Abstract),
                ),
                crate::ast::NodeData::MethodDeclaration(d) => (
                    &d.name,
                    member.has_syntactic_modifier(ModifierFlags::Abstract),
                ),
                crate::ast::NodeData::GetAccessorDeclaration(d) => (
                    &d.name,
                    member.has_syntactic_modifier(ModifierFlags::Abstract),
                ),
                crate::ast::NodeData::SetAccessorDeclaration(d) => (
                    &d.name,
                    member.has_syntactic_modifier(ModifierFlags::Abstract),
                ),
                _ => continue,
            };
            if name_node.kind != SyntaxKind::Identifier {
                continue;
            }
            let name = name_node.text();
            if is_abstract_member {
                if !Self::chain_implements(class, name) {
                    out.push(name.to_string());
                }
            } else if out.iter().any(|m| m == name) {
                out.retain(|m| m != name);
            }
        }
    }

    pub(crate) fn first_return_expression(body: Option<&Arc<Node>>) -> Option<Arc<Node>> {
        fn walk(n: &Arc<Node>) -> Option<Arc<Node>> {
            if let crate::ast::NodeData::ReturnStatement(d) = &n.data
                && let Some(e) = &d.expression
            {
                return Some(Arc::clone(e));
            }
            let mut found: Option<Arc<Node>> = None;
            crate::ast::node_data_generated::for_each_child(n, |child| {
                if found.is_none() {
                    found = walk(child);
                }
                found.is_some()
            });
            found
        }
        body.and_then(walk)
    }

    pub(crate) fn chain_implements(class: &Arc<Node>, name: &str) -> bool {
        for member in Self::class_members_of(class).iter() {
            let (name_node, is_abstract) = match &member.data {
                crate::ast::NodeData::PropertyDeclaration(d) => (
                    &d.name,
                    member.has_syntactic_modifier(ModifierFlags::Abstract),
                ),
                crate::ast::NodeData::MethodDeclaration(d) => (
                    &d.name,
                    member.has_syntactic_modifier(ModifierFlags::Abstract),
                ),
                crate::ast::NodeData::GetAccessorDeclaration(d) => (
                    &d.name,
                    member.has_syntactic_modifier(ModifierFlags::Abstract),
                ),
                crate::ast::NodeData::SetAccessorDeclaration(d) => (
                    &d.name,
                    member.has_syntactic_modifier(ModifierFlags::Abstract),
                ),
                _ => continue,
            };
            if name_node.kind == SyntaxKind::Identifier && name_node.text() == name && !is_abstract
            {
                return true;
            }
        }

        false
    }

    pub(crate) fn assignments_to_name(
        body: &Arc<Node>,
        name: &str,
    ) -> Vec<(crate::core::text::TextRange, Arc<Node>)> {
        let mut found = Vec::new();
        fn walk(
            n: &Arc<Node>,
            name: &str,
            found: &mut Vec<(crate::core::text::TextRange, Arc<Node>)>,
        ) {
            if let crate::ast::NodeData::BinaryExpression(data) = &n.data
                && data.operator_token.kind == SyntaxKind::EqualsToken
                && data.left.kind == SyntaxKind::Identifier
                && data.left.text() == name
            {
                found.push((data.left.loc, Arc::clone(&data.right)));
            }
            crate::ast::node_data_generated::for_each_child(n, |child| {
                walk(child, name, found);
                false
            });
        }
        walk(body, name, &mut found);
        found
    }

    pub fn resolve_qualified_symbol(&mut self, name: &Arc<Node>) -> Option<Arc<Symbol>> {
        match self.resolve_qualified_symbol_traced(name) {
            Ok(s) => Some(s),
            Err(_) => None,
        }
    }

    pub fn resolve_qualified_symbol_traced(
        &mut self,
        name: &Arc<Node>,
    ) -> Result<Arc<Symbol>, (Arc<Node>, String, String)> {
        match &name.data {
            crate::ast::NodeData::Identifier(_) => match self.resolve_identifier(name) {
                Some(s) => Ok(s),
                None => Err((Arc::clone(name), String::new(), String::new())),
            },
            crate::ast::NodeData::QualifiedName(data) => {
                self.resolve_qualified_tail(&data.left, &data.right)
            }

            crate::ast::NodeData::PropertyAccessExpression(pa) => {
                let mut base = &pa.expression;
                while let crate::ast::NodeData::ParenthesizedExpression(p) = &base.data {
                    base = &p.expression;
                }
                if matches!(
                    base.kind,
                    SyntaxKind::Identifier
                        | SyntaxKind::QualifiedName
                        | SyntaxKind::PropertyAccessExpression
                ) {
                    self.resolve_qualified_tail(base, &pa.name)
                } else {
                    Err((Arc::clone(name), String::new(), String::new()))
                }
            }
            _ => Err((Arc::clone(name), String::new(), String::new())),
        }
    }

    pub(crate) fn resolve_qualified_tail(
        &mut self,
        left: &Arc<Node>,
        right: &Arc<Node>,
    ) -> Result<Arc<Symbol>, (Arc<Node>, String, String)> {
        {
            let mut symbol = self.resolve_qualified_symbol_traced(left)?;
            let path_so_far = qualified_name_text(left);
            symbol = self.resolve_alias_base(symbol);

            if symbol.flags == SymbolFlags::Alias
                && let Some(module_sym) = self.resolve_import_alias_module(&symbol)
            {
                symbol = module_sym;
            }

            let text = right.text();
            let mut next = symbol
                .exports
                .get(text)
                .or_else(|| symbol.members.get(text))
                .cloned()
                .or_else(|| self.ambient_namespace_local(&symbol, text))
                .or_else(|| self.object_literal_export_member(&symbol, text));

            if next.is_none()
                && let Some(ea_sym) = symbol.exports.get("export=")
                && let Some(decl) = ea_sym
                    .declarations
                    .iter()
                    .find(|d| d.kind == SyntaxKind::ExportAssignment)
                && let crate::ast::NodeData::ExportAssignment(ea) = &decl.data
                && ea.is_export_equals
                && matches!(
                    ea.expression.kind,
                    SyntaxKind::Identifier | SyntaxKind::QualifiedName
                )
            {
                let scope = symbol
                    .declarations
                    .iter()
                    .find(|d| d.kind == SyntaxKind::ModuleDeclaration)
                    .cloned();
                if let Some(scope) = scope {
                    self.push_scope(&scope);
                    let target = self.resolve_identifier(&ea.expression);
                    self.pop_scope();
                    if let Some(target) = target
                        && target.flags.contains(SymbolFlags::ValueModule)
                    {
                        next = target
                            .exports
                            .get(text)
                            .or_else(|| target.members.get(text))
                            .cloned()
                            .or_else(|| self.ambient_namespace_local(&target, text));
                    }
                }
            }

            let base_is_unresolved_require_alias = symbol.flags == SymbolFlags::Alias
                && symbol.declarations.iter().any(|d| {
                    if let crate::ast::NodeData::ImportEqualsDeclaration(ied) = &d.data
                        && let crate::ast::NodeData::ExternalModuleReference(ext) =
                            &ied.module_reference.data
                        && ext.expression.kind == SyntaxKind::StringLiteral
                    {
                        self.resolve_module_file_symbol(&ext.expression.text())
                            .is_none()
                    } else {
                        false
                    }
                });
            if base_is_unresolved_require_alias {
                return Ok(symbol);
            }
            match next {
                Some(next) => {
                    let resolved = if next.flags.intersects(SymbolFlags::Alias) {
                        let scope = symbol
                            .declarations
                            .iter()
                            .find(|d| {
                                d.kind == SyntaxKind::ModuleDeclaration
                                    || d.kind == SyntaxKind::SourceFile
                            })
                            .cloned();
                        if let Some(ref scope) = scope {
                            self.push_scope(scope);
                        }
                        let base = self.resolve_alias_base(Arc::clone(&next));
                        if scope.is_some() {
                            self.pop_scope();
                        }
                        base
                    } else {
                        match self.follow_alias(&next) {
                            Some(f) => f,
                            None => next,
                        }
                    };
                    Ok(resolved)
                }
                None => {
                    let _ = path_so_far;
                    Err((
                        Arc::clone(right),
                        Self::namespace_full_path(&symbol),
                        text.to_string(),
                    ))
                }
            }
        }
    }

    pub(crate) fn ambient_ancestor(&self, node: &Arc<Node>) -> bool {
        let mut cur = node.parent.as_ref();
        while let Some(a) = cur {
            if a.has_syntactic_modifier(ModifierFlags::Ambient) {
                return true;
            }
            cur = a.parent.as_ref();
        }
        false
    }
}
