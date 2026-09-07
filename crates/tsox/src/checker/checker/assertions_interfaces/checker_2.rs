#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn check_class_function_merge(&mut self, statements: &[Arc<Node>]) {
        let mut groups: std::collections::BTreeMap<String, Vec<Arc<Node>>> =
            std::collections::BTreeMap::new();
        for s in statements {
            match &s.data {
                crate::ast::NodeData::ClassDeclaration(d) => {
                    if let Some(n) = &d.name
                        && n.kind == SyntaxKind::Identifier
                    {
                        groups
                            .entry(n.text().to_string())
                            .or_default()
                            .push(Arc::clone(s));
                    }
                }
                crate::ast::NodeData::FunctionDeclaration(d) => {
                    if let Some(n) = &d.name
                        && n.kind == SyntaxKind::Identifier
                    {
                        groups
                            .entry(n.text().to_string())
                            .or_default()
                            .push(Arc::clone(s));
                    }
                }
                _ => {}
            }
        }
        for (name, decls) in groups {
            let has_non_ambient_class = decls.iter().any(|d| {
                d.kind == SyntaxKind::ClassDeclaration
                    && self.ambient_context_depth == 0
                    && !d.has_syntactic_modifier(ModifierFlags::Ambient)
            });
            let has_function = decls
                .iter()
                .any(|d| d.kind == SyntaxKind::FunctionDeclaration);
            if !(has_non_ambient_class && has_function) {
                continue;
            }
            for d in decls {
                let (name_node, message): (Option<&Arc<Node>>, _) = match &d.data {
                    crate::ast::NodeData::ClassDeclaration(cd) => (
                        cd.name.as_ref(),
                        crate::diagnostics::messages_generated::
                            CLASS_DECLARATION_CANNOT_IMPLEMENT_OVERLOAD_LIST_FOR_0,
                    ),
                    crate::ast::NodeData::FunctionDeclaration(fd) => (
                        fd.name.as_ref(),
                        crate::diagnostics::messages_generated::
                            FUNCTION_WITH_BODIES_CAN_ONLY_MERGE_WITH_CLASSES_THAT_ARE_AMBIENT,
                    ),
                    _ => continue,
                };
                let Some(name_node) = name_node else { continue };
                let file = self.current_file.clone();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    name_node.loc,
                    message,
                    vec![name.clone()],
                ));
            }
        }
    }

    pub(crate) fn check_function_overloads_recursive(&mut self, statements: &[Arc<Node>]) {
        if self
            .current_file
            .as_ref()
            .is_some_and(|f| f.is_declaration_file)
        {
            return;
        }
        self.check_statement_function_overloads(statements);
        self.check_class_function_merge(statements);
        for s in statements {
            match &s.data {
                crate::ast::NodeData::Block(d) => {
                    self.check_function_overloads_recursive(&d.statements.nodes);
                }
                crate::ast::NodeData::ModuleDeclaration(d) => {
                    if d.modifiers
                        .as_ref()
                        .is_some_and(|m| m.modifier_flags.intersects(ModifierFlags::Ambient))
                    {
                        continue;
                    }
                    if let Some(body) = &d.body
                        && matches!(body.kind, SyntaxKind::Block | SyntaxKind::ModuleBlock)
                        && let crate::ast::NodeData::Block(bd) = &body.data
                    {
                        self.check_function_overloads_recursive(&bd.statements.nodes);
                    }
                    if let Some(body) = &d.body
                        && body.kind == SyntaxKind::ModuleBlock
                        && let crate::ast::NodeData::ModuleBlock(bd) = &body.data
                    {
                        self.check_function_overloads_recursive(&bd.statements.nodes);
                    }
                }
                _ => {}
            }
        }
    }

    pub(crate) fn check_statement_function_overloads(&mut self, statements: &[Arc<Node>]) {
        let ambient_context = self.ambient_context_depth > 0
            || self
                .current_file
                .as_ref()
                .is_some_and(|f| f.is_declaration_file);
        let statements: Vec<Arc<Node>> = statements
            .iter()
            .filter(|s| {
                !matches!(s.kind, SyntaxKind::FunctionDeclaration)
                    || !(ambient_context || s.has_syntactic_modifier(ModifierFlags::Ambient))
            })
            .cloned()
            .collect();
        let mut groups: std::collections::BTreeMap<String, Vec<usize>> =
            std::collections::BTreeMap::new();
        for (idx, s) in statements.iter().enumerate() {
            if s.kind != SyntaxKind::FunctionDeclaration {
                continue;
            }
            if let crate::ast::NodeData::FunctionDeclaration(d) = &s.data
                && let Some(n) = &d.name
                && n.kind == SyntaxKind::Identifier
            {
                groups.entry(n.text().to_string()).or_default().push(idx);
            }
        }
        for (_, idxs) in groups {
            let mut prev: Option<usize> = None;
            let mut has_body = false;
            for &idx in &idxs {
                let body = matches!(
                    &statements[idx].data,
                    crate::ast::NodeData::FunctionDeclaration(d) if d.body.is_some()
                );
                if !body {
                    if let Some(p) = prev {
                        if p + 1 != idx {
                            self.report_function_impl_expected(&statements, p);
                        }
                    }
                } else {
                    has_body = true;
                }
                prev = Some(idx);
            }
            if !has_body {
                let last = idxs[idxs.len() - 1];
                if !statements[last].has_syntactic_modifier(ModifierFlags::Ambient) {
                    self.report_function_impl_expected(&statements, last);
                }
            } else {
                let fn_params = |f: &Arc<Node>| -> (usize, bool) {
                    if let crate::ast::NodeData::FunctionDeclaration(d) = &f.data {
                        let mut rest = false;
                        for p in d.parameters.iter() {
                            if p.kind == SyntaxKind::Parameter {
                                if let crate::ast::NodeData::ParameterDeclaration(pd) = &p.data {
                                    if pd.dot_dot_dot_token.is_some() {
                                        rest = true;
                                        break;
                                    }

                                    let _ = pd.question_token.is_none();
                                }
                            }
                        }
                        (d.parameters.nodes.len(), rest)
                    } else {
                        (0, false)
                    }
                };
                let impl_idx = idxs
                    .iter()
                    .copied()
                    .find(|&i| {
                        matches!(
                            &statements[i].data,
                            crate::ast::NodeData::FunctionDeclaration(d) if d.body.is_some()
                        )
                    })
                    .unwrap_or_else(|| idxs[idxs.len() - 1]);
                let (_impl_total, impl_rest) = fn_params(&statements[impl_idx]);
                let impl_required = {
                    let mut n = 0;
                    if let crate::ast::NodeData::FunctionDeclaration(d) = &statements[impl_idx].data
                    {
                        for p in d.parameters.iter() {
                            if let crate::ast::NodeData::ParameterDeclaration(pd) = &p.data
                                && pd.dot_dot_dot_token.is_none()
                                && pd.question_token.is_none()
                            {
                                n += 1;
                            }
                        }
                    }
                    n
                };
                if !impl_rest {
                    let mut seen_shapes: Vec<String> = Vec::new();
                    for &i in &idxs {
                        if i == impl_idx {
                            continue;
                        }
                        let (overload_count, _) = fn_params(&statements[i]);
                        let shape = if let crate::ast::NodeData::FunctionDeclaration(d) =
                            &statements[i].data
                        {
                            let mut parts = Vec::new();
                            for p in d.parameters.iter() {
                                if let crate::ast::NodeData::ParameterDeclaration(pd) = &p.data {
                                    let t = pd
                                        .type_node
                                        .as_ref()
                                        .map(|tn| tn.text())
                                        .unwrap_or_default();
                                    parts.push(format!(
                                        "{t}{}",
                                        if pd.question_token.is_some() { "?" } else { "" }
                                    ));
                                }
                            }
                            let ret = d.type_node.as_ref().map(|tn| tn.text()).unwrap_or_default();
                            format!("({})=>{}", parts.join(","), ret)
                        } else {
                            String::new()
                        };
                        if seen_shapes.contains(&shape) {
                            continue;
                        }
                        seen_shapes.push(shape);
                        let arity_bad = !impl_rest && overload_count < impl_required;
                        let overload_node = Arc::clone(&statements[i]);
                        let impl_node = Arc::clone(&statements[impl_idx]);
                        let compat = self.overload_signature_compatible_with_implementation(
                            &overload_node,
                            &impl_node,
                        );
                        if (arity_bad || !compat)
                            && let crate::ast::NodeData::FunctionDeclaration(d) =
                                &statements[i].data
                            && let Some(n) = &d.name
                        {
                            let file = self.current_file.clone();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                n.loc,
                                crate::diagnostics::messages_generated::
                                    THIS_OVERLOAD_SIGNATURE_IS_NOT_COMPATIBLE_WITH_ITS_IMPLEMENTATION_SIGNATURE,
                                Vec::new(),
                            ));
                        }
                    }
                }
            }
        }
    }
}
