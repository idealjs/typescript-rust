#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn namespace_member_recursive(
        &mut self,
        namespace: &Arc<Symbol>,
        name: &str,
    ) -> Option<Arc<Symbol>> {
        if let Some(s) = namespace
            .exports
            .get(name)
            .or_else(|| namespace.members.get(name))
        {
            return Some(Arc::clone(s));
        }
        for d in &namespace.declarations {
            if d.kind == SyntaxKind::ModuleDeclaration
                && let Some(s) = self
                    .program
                    .symbol_map()
                    .locals
                    .get(&d.id())
                    .and_then(|l| l.get(name))
            {
                return Some(Arc::clone(s));
            }
        }

        let export_equals = namespace.exports.get("export=")?;
        for d in &export_equals.declarations {
            if let crate::ast::NodeData::ExportAssignment(ea) = &d.data
                && ea.is_export_equals
            {
                if let crate::ast::NodeData::ObjectLiteralExpression(ol) = &ea.expression.data {
                    for prop in ol.properties.iter() {
                        if prop.text() == name
                            && let Some(s) = self.program.symbol_map().symbol_of(prop)
                        {
                            return Some(Arc::clone(s));
                        }
                    }
                    continue;
                }
                if matches!(
                    ea.expression.kind,
                    SyntaxKind::Identifier | SyntaxKind::QualifiedName
                ) {
                    let scope_decl = namespace
                        .declarations
                        .iter()
                        .find(|d| d.kind == SyntaxKind::ModuleDeclaration)
                        .cloned();
                    let target = scope_decl.and_then(|scope_decl| {
                        self.push_scope(&scope_decl);
                        let t = self.resolve_qualified_symbol(&ea.expression);
                        self.pop_scope();
                        t
                    });
                    if let Some(mut target) = target {
                        for _ in 0..4 {
                            if target.flags.contains(SymbolFlags::ValueModule) {
                                break;
                            }
                            if target.flags != SymbolFlags::Alias {
                                break;
                            }
                            let next = target
                                .declarations
                                .iter()
                                .find(|d| d.kind == SyntaxKind::ImportEqualsDeclaration)
                                .and_then(|d| {
                                    if let crate::ast::NodeData::ImportEqualsDeclaration(ied) =
                                        &d.data
                                        && matches!(
                                            ied.module_reference.kind,
                                            SyntaxKind::Identifier | SyntaxKind::QualifiedName
                                        )
                                    {
                                        Some(self.resolve_qualified_symbol(&ied.module_reference))
                                    } else {
                                        None
                                    }
                                })
                                .flatten();
                            match next {
                                Some(n) => target = n,
                                None => break,
                            }
                        }
                        if target.flags.contains(SymbolFlags::ValueModule) {
                            return self.namespace_member_recursive(&target, name);
                        }
                        return Some(target);
                    }
                }
            }
        }
        None
    }

    pub(crate) fn namespace_full_path(symbol: &Arc<Symbol>) -> String {
        let decl = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ModuleDeclaration);
        let Some(decl) = decl else {
            return symbol.name.clone();
        };
        let mut parts: Vec<String> = Vec::new();
        let mut current: Option<&Arc<Node>> = Some(decl);
        while let Some(n) = current {
            if let crate::ast::NodeData::ModuleDeclaration(md) = &n.data {
                parts.push(md.name.text().trim_matches(['"', '\'']).to_string());
            }
            current = n.parent.as_ref();
        }
        parts.reverse();
        parts.join(".")
    }
}
