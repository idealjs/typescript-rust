use std::sync::Arc;

use crate::ast::{
    ModifierFlags, Node, SymbolFlags, SyntaxKind,
};







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
            SourceFile | ModuleDeclaration | Block | CaseBlock | ForStatement
            | ForInStatement | ForOfStatement => out.push(Arc::clone(node)),
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
                !local.flags.intersects(variable_bits)
                    || reference_kinds.intersects(variable_bits)
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
                                if !variable_parents
                                    .iter()
                                    .any(|(n, _)| Arc::ptr_eq(n, parent))
                                {
                                    variable_parents.push((Arc::clone(parent), false));
                                }
                            }
                        }
                    }
                    SyntaxKind::ImportClause
                    | SyntaxKind::ImportSpecifier
                    | SyntaxKind::NamespaceImport => {
                        if !Self::name_starts_with_underscore(declaration) {
                            let clause =
                                Self::import_clause_from_imported(declaration);
                            match import_clauses
                                .iter_mut()
                                .find(|(c, _)| Arc::ptr_eq(c, &clause))
                            {
                                Some((_, v)) => v.push(Arc::clone(declaration)),
                                None => import_clauses
                                    .push((clause, vec![Arc::clone(declaration)])),
                            }
                        }
                    }
                    _ => {
                        if declaration.kind != SyntaxKind::TypeParameter
                            && declaration.kind != SyntaxKind::ModuleDeclaration
                        {
                            let name = local.name.clone();
                            let is_type_decl =
                                matches!(declaration.kind, SyntaxKind::TypeAliasDeclaration
                                    | SyntaxKind::InterfaceDeclaration
                                    | SyntaxKind::ClassDeclaration
                                    | SyntaxKind::EnumDeclaration);
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
            SyntaxKind::NamespaceImport => node
                .parent
                .clone()
                .unwrap_or_else(|| Arc::clone(node)),
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
            crate::ast::NodeData::FunctionDeclaration(d) => {
                d.parameters.iter().cloned().collect()
            }
            crate::ast::NodeData::FunctionExpression(d) => {
                d.parameters.iter().cloned().collect()
            }
            crate::ast::NodeData::ArrowFunction(d) => d.parameters.iter().cloned().collect(),
            crate::ast::NodeData::MethodDeclaration(d) => {
                d.parameters.iter().cloned().collect()
            }
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
                    let is_pattern = n
                        .as_ref()
                        .is_some_and(|n| Self::is_binding_pattern(n));
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

    pub(crate) fn report_unused_binding_elements(&mut self, pattern: &Arc<Node>) {
        let elements: Vec<Arc<Node>> = match &pattern.data {
            crate::ast::NodeData::BindingPattern(d) => {
                d.elements.iter().cloned().collect()
            }
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
                crate::ast::NodeData::BindingPattern(d) => {
                    d.elements.iter().cloned().collect()
                }
                _ => return true,
            };
            return elements
                .iter()
                .all(|e| self.is_unreferenced_variable_declaration(e));
        }

        if let Some(sym) = self
            .program
            .symbol_map()
            .symbol_of(node)
        {
            if let Some(kinds) = self.symbol_reference_kinds.get(&sym.id()) {
                if kinds
                    .intersects(SymbolFlags::FunctionScopedVariable | SymbolFlags::BlockScopedVariable)
                {
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
                    let is_last = elements
                        .last()
                        .is_some_and(|last| Arc::ptr_eq(last, node));
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
                    crate::ast::NodeData::NamedImports(d) => {
                        d.elements.iter().cloned().collect()
                    }
                    _ => Vec::new(),
                };
                declaration_count += elements.len();
            }
        }
        if declaration_count > 1 && declaration_count == unused.len() {
            let loc = clause
                .parent
                .as_ref()
                .map(|p| p.loc)
                .unwrap_or(clause.loc);
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
