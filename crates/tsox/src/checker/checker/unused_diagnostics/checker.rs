#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn check_unused_identifiers_in_file(&mut self, file_node: &Arc<Node>) {
        let no_locals = !self.compiler_options.no_unused_locals.is_true();
        let no_params = !self.compiler_options.no_unused_parameters.is_true();
        if no_locals && no_params {
            return;
        }
        let mut containers: Vec<Arc<Node>> = Vec::new();
        Self::collect_unused_check_containers(file_node, &mut containers);
        for container in containers {
            self.check_unused_locals_and_parameters(&container);
        }
    }

    pub(crate) fn collect_unused_check_containers(node: &Arc<Node>, out: &mut Vec<Arc<Node>>) {
        use SyntaxKind::*;
        match node.kind {
            SourceFile | ModuleDeclaration | Block | CaseBlock | ForStatement | ForInStatement
            | ForOfStatement => out.push(Arc::clone(node)),
            Constructor | FunctionExpression | FunctionDeclaration | ArrowFunction
            | MethodDeclaration | GetAccessor | SetAccessor => {
                if Self::function_like_has_body(node) {
                    out.push(Arc::clone(node));
                }
            }
            _ => {}
        }
        crate::ast::node_data_generated::for_each_child(node, |child| {
            Self::collect_unused_check_containers(child, out);
            false
        });
    }

    pub(crate) fn function_like_has_body(node: &Arc<Node>) -> bool {
        use crate::ast::NodeData;
        match &node.data {
            NodeData::ConstructorDeclaration(d) => d.body.is_some(),
            NodeData::FunctionDeclaration(d) => d.body.is_some(),

            NodeData::FunctionExpression(_) | NodeData::ArrowFunction(_) => true,
            NodeData::MethodDeclaration(d) => d.body.is_some(),
            NodeData::GetAccessorDeclaration(d) => d.body.is_some(),
            NodeData::SetAccessorDeclaration(d) => d.body.is_some(),
            _ => false,
        }
    }

    pub(crate) fn check_unused_locals_and_parameters(&mut self, container: &Arc<Node>) {
        let Some(locals) = self.program.symbol_map().locals.get(&container.id()) else {
            return;
        };
        let locals: Vec<Arc<crate::ast::Symbol>> = locals.entries.values().cloned().collect();

        let mut variable_parents: Vec<(Arc<Node>, bool)> = Vec::new();

        let mut import_clauses: Vec<(Arc<Node>, Vec<Arc<Node>>)> = Vec::new();
        for local in locals {
            let reference_kinds = self
                .symbol_reference_kinds
                .get(&local.id())
                .map(|e| *e)
                .unwrap_or(SymbolFlags::empty());

            let variable_bits =
                SymbolFlags::FunctionScopedVariable | SymbolFlags::BlockScopedVariable;
            let skip = if local.flags.contains(SymbolFlags::TypeParameter) {
                !local.flags.intersects(variable_bits) || reference_kinds.intersects(variable_bits)
            } else {
                reference_kinds != SymbolFlags::empty()
                    || local.export_symbol.is_some()
                    || local.flags.contains(SymbolFlags::ModuleExports)
            };
            if skip {
                continue;
            }
            for declaration in &local.declarations {
                match declaration.kind {
                    SyntaxKind::VariableDeclaration
                    | SyntaxKind::Parameter
                    | SyntaxKind::BindingElement => {
                        if let Some(root) = Self::root_declaration(declaration) {
                            if let Some(parent) = root.parent.as_ref() {
                                if !variable_parents.iter().any(|(n, _)| Arc::ptr_eq(n, parent)) {
                                    variable_parents.push((Arc::clone(parent), false));
                                }
                            }
                        }
                    }
                    SyntaxKind::ImportClause
                    | SyntaxKind::ImportSpecifier
                    | SyntaxKind::NamespaceImport => {
                        if !Self::name_starts_with_underscore(declaration) {
                            let clause = Self::import_clause_from_imported(declaration);
                            match import_clauses
                                .iter_mut()
                                .find(|(c, _)| Arc::ptr_eq(c, &clause))
                            {
                                Some((_, v)) => v.push(Arc::clone(declaration)),
                                None => {
                                    import_clauses.push((clause, vec![Arc::clone(declaration)]))
                                }
                            }
                        }
                    }
                    _ => {
                        if declaration.kind != SyntaxKind::TypeParameter
                            && declaration.kind != SyntaxKind::ModuleDeclaration
                        {
                            let name = local.name.clone();
                            let is_type_decl = matches!(
                                declaration.kind,
                                SyntaxKind::TypeAliasDeclaration
                                    | SyntaxKind::InterfaceDeclaration
                                    | SyntaxKind::ClassDeclaration
                                    | SyntaxKind::EnumDeclaration
                            );
                            self.report_unused_local(declaration, &name, is_type_decl);
                        }
                    }
                }
            }
        }
        for (parent, _is_param) in variable_parents {
            if parent.kind == SyntaxKind::VariableDeclarationList {
                self.report_unused_variables(&parent);
            } else {
                self.report_unused_parameters(&parent);
            }
        }
        for (clause, unused) in import_clauses {
            self.report_unused_imports(&clause, &unused);
        }
    }

    pub(crate) fn root_declaration(node: &Arc<Node>) -> Option<Arc<Node>> {
        let mut cursor = Arc::clone(node);
        for _ in 0..100 {
            match cursor.kind {
                SyntaxKind::BindingElement => {
                    cursor = cursor.parent.as_ref()?.clone();
                }
                SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern => {
                    cursor = cursor.parent.as_ref()?.clone();
                }
                _ => return Some(cursor),
            }
        }
        None
    }

    pub(crate) fn name_starts_with_underscore(node: &Arc<Node>) -> bool {
        let text = node.text();
        !text.is_empty() && text.starts_with('_')
    }

    pub(crate) fn import_clause_from_imported(node: &Arc<Node>) -> Arc<Node> {
        match node.kind {
            SyntaxKind::ImportClause => Arc::clone(node),
            SyntaxKind::NamespaceImport => node.parent.clone().unwrap_or_else(|| Arc::clone(node)),
            _ => node
                .parent
                .clone()
                .and_then(|p| p.parent.clone())
                .unwrap_or_else(|| Arc::clone(node)),
        }
    }

    pub(crate) fn report_unused_local(&mut self, node: &Arc<Node>, name: &str, is_type_decl: bool) {
        let message: &'static crate::diagnostics::Message = if is_type_decl {
            &crate::diagnostics::messages_generated::X_0_IS_DECLARED_BUT_NEVER_USED
        } else {
            &crate::diagnostics::messages_generated::X_0_IS_DECLARED_BUT_ITS_VALUE_IS_NEVER_READ
        };
        let loc = Self::name_or_node_loc(node);
        let is_param = node.kind == SyntaxKind::Parameter;
        self.report_unused(node, is_param, loc, message, vec![name.to_string()]);
    }

    pub(crate) fn report_unused_variables(&mut self, list: &Arc<Node>) {
        let declarations: Vec<Arc<Node>> = match &list.data {
            crate::ast::NodeData::VariableDeclarationList(d) => {
                d.declarations.iter().cloned().collect()
            }
            _ => return,
        };
        if declarations.len() > 1
            && declarations
                .iter()
                .all(|d| self.is_unreferenced_variable_declaration(d))
        {
            self.report_unused(
                list,
                false,
                list.loc,
                &crate::diagnostics::messages_generated::ALL_VARIABLES_ARE_UNUSED,
                vec![],
            );
        } else {
            self.report_unused_variable_declarations(&declarations);
        }
    }

    pub(crate) fn report_unused_parameters(&mut self, function: &Arc<Node>) {
        let parameters: Vec<Arc<Node>> = match &function.data {
            crate::ast::NodeData::ConstructorDeclaration(d) => {
                d.parameters.iter().cloned().collect()
            }
            crate::ast::NodeData::FunctionDeclaration(d) => d.parameters.iter().cloned().collect(),
            crate::ast::NodeData::FunctionExpression(d) => d.parameters.iter().cloned().collect(),
            crate::ast::NodeData::ArrowFunction(d) => d.parameters.iter().cloned().collect(),
            crate::ast::NodeData::MethodDeclaration(d) => d.parameters.iter().cloned().collect(),
            crate::ast::NodeData::GetAccessorDeclaration(d) => {
                d.parameters.iter().cloned().collect()
            }
            crate::ast::NodeData::SetAccessorDeclaration(d) => {
                d.parameters.iter().cloned().collect()
            }
            _ => return,
        };
        self.report_unused_variable_declarations(&parameters);
    }

    pub(crate) fn report_unused_variable_declarations(&mut self, declarations: &[Arc<Node>]) {
        for declaration in declarations {
            let (name_node, is_pattern) = match &declaration.data {
                crate::ast::NodeData::VariableDeclaration(d) => {
                    (Some(Arc::clone(&d.name)), Self::is_binding_pattern(&d.name))
                }
                crate::ast::NodeData::ParameterDeclaration(d) => {
                    (Some(Arc::clone(&d.name)), Self::is_binding_pattern(&d.name))
                }
                crate::ast::NodeData::BindingElement(d) => {
                    let n = d.name.clone();
                    let is_pattern = n.as_ref().is_some_and(|n| Self::is_binding_pattern(n));
                    (n, is_pattern)
                }
                _ => continue,
            };
            let Some(name_node) = name_node else { continue };

            if declaration.kind == SyntaxKind::Parameter {
                if let crate::ast::NodeData::ParameterDeclaration(d) = &declaration.data {
                    if d.modifiers.as_ref().is_some_and(|m| {
                        m.modifier_flags.intersects(
                            ModifierFlags::Public
                                | ModifierFlags::Private
                                | ModifierFlags::Protected
                                | ModifierFlags::Readonly,
                        )
                    }) {
                        continue;
                    }
                    if name_node.kind == SyntaxKind::ThisKeyword {
                        continue;
                    }
                }
            }
            if is_pattern {
                self.report_unused_binding_elements(&name_node);
            } else if self.is_unreferenced_variable_declaration(declaration) {
                let name = name_node.text().to_string();
                self.report_unused(
                    declaration,
                    declaration.kind == SyntaxKind::Parameter,
                    name_node.loc,
                    &crate::diagnostics::messages_generated::
                        X_0_IS_DECLARED_BUT_ITS_VALUE_IS_NEVER_READ,
                    vec![name],
                );
            }
        }
    }
}
