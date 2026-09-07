#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn report_unused_binding_elements(&mut self, pattern: &Arc<Node>) {
        let elements: Vec<Arc<Node>> = match &pattern.data {
            crate::ast::NodeData::BindingPattern(d) => d.elements.iter().cloned().collect(),
            _ => return,
        };
        if elements.len() > 1
            && elements
                .iter()
                .all(|e| self.is_unreferenced_variable_declaration(e))
        {
            self.report_unused(
                pattern,
                false,
                pattern.loc,
                &crate::diagnostics::messages_generated::ALL_DESTRUCTURED_ELEMENTS_ARE_UNUSED,
                vec![],
            );
        } else {
            self.report_unused_variable_declarations(&elements);
        }
    }

    pub(crate) fn is_unreferenced_variable_declaration(&self, node: &Arc<Node>) -> bool {
        let name_node = match &node.data {
            crate::ast::NodeData::VariableDeclaration(d) => Some(Arc::clone(&d.name)),
            crate::ast::NodeData::ParameterDeclaration(d) => Some(Arc::clone(&d.name)),
            crate::ast::NodeData::BindingElement(d) => d.name.clone(),
            _ => return true,
        };
        let Some(name_node) = name_node else {
            return true;
        };
        if Self::is_binding_pattern(&name_node) {
            let elements: Vec<Arc<Node>> = match &name_node.data {
                crate::ast::NodeData::BindingPattern(d) => d.elements.iter().cloned().collect(),
                _ => return true,
            };
            return elements
                .iter()
                .all(|e| self.is_unreferenced_variable_declaration(e));
        }

        if let Some(sym) = self.program.symbol_map().symbol_of(node) {
            if let Some(kinds) = self.symbol_reference_kinds.get(&sym.id()) {
                if kinds.intersects(
                    SymbolFlags::FunctionScopedVariable | SymbolFlags::BlockScopedVariable,
                ) {
                    return false;
                }
            }
        }

        if node.kind == SyntaxKind::BindingElement {
            if let Some(parent) = node.parent.as_ref() {
                if parent.kind == SyntaxKind::ObjectBindingPattern {
                    let elements: Vec<Arc<Node>> = match &parent.data {
                        crate::ast::NodeData::BindingPattern(d) => {
                            d.elements.iter().cloned().collect()
                        }
                        _ => Vec::new(),
                    };
                    let is_last = elements.last().is_some_and(|last| Arc::ptr_eq(last, node));
                    let last_has_dots = elements.last().is_some_and(|last| {
                        matches!(&last.data,
                            crate::ast::NodeData::BindingElement(d) if d.dot_dot_dot_token.is_some())
                    });
                    let has_property_name = matches!(&node.data,
                        crate::ast::NodeData::BindingElement(d) if d.property_name.is_some());
                    if !is_last && last_has_dots && !has_property_name {
                        return false;
                    }
                }
            }
        }

        let underscore_exempt = match node.kind {
            SyntaxKind::Parameter => true,
            SyntaxKind::VariableDeclaration => {
                let mut in_for = false;
                if let Some(parent) = node.parent.as_ref() {
                    if parent.kind == SyntaxKind::VariableDeclarationList {
                        if let Some(gp) = parent.parent.as_ref() {
                            in_for = matches!(
                                gp.kind,
                                SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement
                            );
                        }
                    }
                }
                in_for || node.flags.contains(crate::ast::NodeFlags::Using)
            }
            SyntaxKind::BindingElement => {
                let parent_is_object_pattern = node
                    .parent
                    .as_ref()
                    .is_some_and(|p| p.kind == SyntaxKind::ObjectBindingPattern);
                let has_property_name = matches!(&node.data,
                    crate::ast::NodeData::BindingElement(d) if d.property_name.is_some());
                !(parent_is_object_pattern && !has_property_name)
            }
            _ => false,
        };
        if underscore_exempt && Self::name_node_starts_with_underscore(&name_node) {
            return false;
        }
        true
    }

    pub(crate) fn name_node_starts_with_underscore(node: &Arc<Node>) -> bool {
        let text = node.text();
        !text.is_empty() && text.starts_with('_')
    }

    pub(crate) fn is_binding_pattern(node: &Arc<Node>) -> bool {
        matches!(
            node.kind,
            SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern
        )
    }

    pub(crate) fn name_or_node_loc(node: &Arc<Node>) -> crate::core::text::TextRange {
        crate::ast::utilities::get_name_of_declaration(node)
            .map(|n| n.loc)
            .unwrap_or(node.loc)
    }

    pub(crate) fn report_unused_imports(&mut self, clause: &Arc<Node>, unused: &[Arc<Node>]) {
        let mut declaration_count = 0usize;
        let named_bindings: Option<Arc<Node>> = match &clause.data {
            crate::ast::NodeData::ImportClause(d) => {
                if d.name.is_some() {
                    declaration_count += 1;
                }
                d.named_bindings.clone()
            }
            _ => None,
        };
        if let Some(nb) = &named_bindings {
            if nb.kind == SyntaxKind::NamespaceImport {
                declaration_count += 1;
            } else {
                let elements: Vec<Arc<Node>> = match &nb.data {
                    crate::ast::NodeData::NamedImports(d) => d.elements.iter().cloned().collect(),
                    _ => Vec::new(),
                };
                declaration_count += elements.len();
            }
        }
        if declaration_count > 1 && declaration_count == unused.len() {
            let loc = clause.parent.as_ref().map(|p| p.loc).unwrap_or(clause.loc);
            self.report_unused(
                clause,
                false,
                loc,
                &crate::diagnostics::messages_generated::
                    ALL_IMPORTS_IN_IMPORT_DECLARATION_ARE_UNUSED,
                vec![],
            );
        } else {
            for u in unused {
                let name = u.text().to_string();
                let is_type_decl = false;
                self.report_unused_local(u, &name, is_type_decl);
            }
        }
    }

    pub(crate) fn report_unused(
        &mut self,
        location: &Arc<Node>,
        is_parameter: bool,
        loc: crate::core::text::TextRange,
        message: &'static crate::diagnostics::Message,
        args: Vec<String>,
    ) {
        let ambient = location.flags.contains(crate::ast::NodeFlags::Ambient)
            || self.ambient_ancestor(location)
            || self
                .get_source_file_of_node(location)
                .is_some_and(|f| f.is_declaration_file);
        if ambient {
            return;
        }
        let is_error = if is_parameter {
            self.compiler_options.no_unused_parameters.is_true()
        } else {
            self.compiler_options.no_unused_locals.is_true()
        };
        if !is_error {
            return;
        }
        let file = self.current_file.clone();
        self.diagnostics
            .add(crate::ast::Diagnostic::new(file, loc, *message, args));
    }

    pub(crate) fn set_parent_pointers(&mut self, node: &Arc<Node>) {
        use crate::ast::node_data_generated::for_each_child;

        let mut children: Vec<Arc<Node>> = Vec::new();
        for_each_child(node, |child| {
            children.push(Arc::clone(child));
            false
        });
        let parent_clone = Arc::clone(node);
        for child in &children {
            let child_mut = Arc::as_ptr(child) as *mut Node;

            let already = unsafe {
                (*child_mut)
                    .parent
                    .as_ref()
                    .map_or(false, |p| Arc::ptr_eq(p, &parent_clone))
            };
            if !already {
                unsafe {
                    (*child_mut).parent = Some(Arc::clone(&parent_clone));
                }
            }
            self.set_parent_pointers(child);
        }
    }
}
