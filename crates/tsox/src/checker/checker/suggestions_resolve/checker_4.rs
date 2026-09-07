#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn check_duplicate_function_implementations(&mut self, node: &Arc<Node>) {
        let crate::ast::NodeData::FunctionDeclaration(data) = &node.data else {
            return;
        };
        let Some(name) = &data.name else { return };
        if name.kind != SyntaxKind::Identifier {
            return;
        }
        let Some(parent) = node.parent.as_ref() else {
            return;
        };
        let stmts = match &parent.data {
            crate::ast::NodeData::SourceFile(sf) => Some(&sf.statements),
            crate::ast::NodeData::ModuleBlock(mb) => Some(&mb.statements),
            _ => None,
        };
        let Some(stmts) = stmts else {
            return;
        };
        let is_ambient = node.flags.contains(NodeFlags::Ambient)
            || self
                .current_file
                .as_ref()
                .is_some_and(|f| f.is_declaration_file);
        let fns: Vec<&Arc<Node>> = stmts
            .iter()
            .filter(|s| {
                s.kind == SyntaxKind::FunctionDeclaration
                    && matches!(&s.data, crate::ast::NodeData::FunctionDeclaration(d) if d
                        .name
                        .as_ref()
                        .is_some_and(|n| n.text() == name.text()))
            })
            .collect();

        if fns.first().is_none_or(|first| !Arc::ptr_eq(first, node)) {
            return;
        }
        let bodied = fns
            .iter()
            .filter(|f| {
                matches!(&f.data, crate::ast::NodeData::FunctionDeclaration(d) if d.body.is_some())
            })
            .count();
        let file = self.current_file.clone();
        if bodied >= 2 && !is_ambient {
            for f in &fns {
                if let crate::ast::NodeData::FunctionDeclaration(d) = &f.data
                    && let Some(fname) = &d.name
                {
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file.clone(),
                        fname.loc,
                        crate::diagnostics::messages_generated::DUPLICATE_FUNCTION_IMPLEMENTATION,
                        vec![],
                    ));
                }
            }
        }

        let is_ambient_decl = |f: &Arc<Node>| {
            f.has_syntactic_modifier(ModifierFlags::Ambient) || f.flags.contains(NodeFlags::Ambient)
        };
        let canonical = fns
            .iter()
            .find(|f| {
                matches!(&f.data, crate::ast::NodeData::FunctionDeclaration(d) if d.body.is_some())
            })
            .or_else(|| fns.first());
        if let Some(canonical) = canonical {
            let canonical_ambient = is_ambient_decl(canonical);
            for f in &fns {
                let has_body = matches!(&f.data, crate::ast::NodeData::FunctionDeclaration(d) if d.body.is_some());
                if !has_body && is_ambient_decl(f) != canonical_ambient {
                    if let crate::ast::NodeData::FunctionDeclaration(d) = &f.data
                        && let Some(fname) = &d.name
                    {
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file.clone(),
                            fname.loc,
                            crate::diagnostics::messages_generated::
                                OVERLOAD_SIGNATURES_MUST_ALL_BE_AMBIENT_OR_NON_AMBIENT,
                            vec![],
                        ));
                    }
                }
            }
        }
    }

    pub(crate) fn check_overload_implementation_follows(&mut self, node: &Arc<Node>) {
        let crate::ast::NodeData::FunctionDeclaration(data) = &node.data else {
            return;
        };
        if data.body.is_some() {
            return;
        }
        let Some(name) = &data.name else { return };
        if name.kind != SyntaxKind::Identifier {
            return;
        }
        let Some(parent) = node.parent.as_ref() else {
            return;
        };
        let stmts = match &parent.data {
            crate::ast::NodeData::SourceFile(sf) => Some(&sf.statements),
            crate::ast::NodeData::ModuleBlock(mb) => Some(&mb.statements),
            _ => None,
        };
        let Some(stmts) = stmts else { return };
        let is_ambient = node.has_syntactic_modifier(ModifierFlags::Ambient)
            || node.flags.contains(NodeFlags::Ambient)
            || self.ambient_context_depth > 0
            || node.parent.as_ref().is_some_and(|_| {
                let mut anc = node.parent.as_ref();
                let mut found = false;
                while let Some(a) = anc {
                    if a.has_syntactic_modifier(ModifierFlags::Ambient) {
                        found = true;
                        break;
                    }
                    anc = a.parent.as_ref();
                }
                found
            })
            || self
                .current_file
                .as_ref()
                .is_some_and(|f| f.is_declaration_file);
        if is_ambient {
            return;
        }
        let next = stmts.iter().enumerate().find_map(|(i, s)| {
            if Arc::ptr_eq(s, node) {
                stmts.nodes.get(i + 1).cloned()
            } else {
                None
            }
        });

        if next.as_ref().is_some_and(|n| {
            matches!(&n.data, crate::ast::NodeData::FunctionDeclaration(d) if d
                .name
                .as_ref()
                .is_some_and(|n2| n2.text() == name.text()))
        }) {
            return;
        }

        if let Some(n) = &next
            && matches!(&n.data, crate::ast::NodeData::FunctionDeclaration(d) if d.body.is_some())
            && n.kind == SyntaxKind::FunctionDeclaration
        {
            if let crate::ast::NodeData::FunctionDeclaration(d) = &n.data
                && let Some(next_name) = &d.name
                && next_name.kind == SyntaxKind::Identifier
                && next_name.text() != name.text()
            {
                let already = self
                    .diagnostics
                    .get_all()
                    .iter()
                    .any(|d| d.code == 2389 && d.loc == next_name.loc);
                if !already {
                    let file = self.current_file.clone();
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        next_name.loc,
                        crate::diagnostics::messages_generated::
                            FUNCTION_IMPLEMENTATION_NAME_MUST_BE_0,
                        vec![name.text().to_string()],
                    ));
                }
                return;
            }
        }

        let already = self
            .diagnostics
            .get_all()
            .iter()
            .any(|d| d.code == 2391 && d.loc == name.loc);
        if !already {
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                name.loc,
                crate::diagnostics::messages_generated::
                    FUNCTION_IMPLEMENTATION_IS_MISSING_OR_NOT_IMMEDIATELY_FOLLOWING_THE_DECLARATION,
                vec![],
            ));
        }
    }

    pub(crate) fn check_multiple_constructor_implementations(&mut self, node: &Arc<Node>) {
        let Some(class) = node.parent.as_ref() else {
            return;
        };
        if !matches!(
            class.kind,
            SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
        ) {
            return;
        }
        let crate::ast::NodeData::ClassDeclaration(cd) = &class.data else {
            return;
        };
        let ctors: Vec<&Arc<Node>> = cd
            .members
            .iter()
            .filter(|m| m.kind == SyntaxKind::Constructor)
            .collect();
        if ctors.first().is_none_or(|first| !Arc::ptr_eq(first, node)) {
            return;
        }
        let bodied = ctors
            .iter()
            .filter(|c| {
                matches!(&c.data, crate::ast::NodeData::ConstructorDeclaration(d) if d.body.is_some())
            })
            .count();
        if bodied < 2 {
            return;
        }
        let file = self.current_file.clone();
        for ctor in ctors {
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file.clone(),
                ctor.loc,
                crate::diagnostics::messages_generated::
                    MULTIPLE_CONSTRUCTOR_IMPLEMENTATIONS_ARE_NOT_ALLOWED,
                vec![],
            ));
        }
    }
}
