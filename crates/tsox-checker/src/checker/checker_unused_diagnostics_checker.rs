#![allow(unused_imports)]

use crate::checker::checker_unused_diagnostics::*;

impl Checker {
    pub(crate) fn check_unused_identifiers_in_file(&mut self, file_node: &Arc<Node>) {
        let no_locals = !self.compiler_options.no_unused_locals.is_true();
        let no_params = !self.compiler_options.no_unused_parameters.is_true();
        if no_locals && no_params {
            return;
        }
        let mut containers: Vec<(Arc<Node>, bool)> = Vec::new();
        Self::collect_unused_check_containers(file_node, &mut containers);
        for (container, check_locals) in containers {
            if check_locals {
                self.check_unused_locals_and_parameters(&container);
            }
            if matches!(
                container.kind,
                SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
            ) {
                self.check_unused_class_members(&container);
            }
            self.check_unused_type_parameters(&container);
        }
    }

    pub(crate) fn collect_unused_check_containers(
        node: &Arc<Node>,
        out: &mut Vec<(Arc<Node>, bool)>,
    ) {
        use SyntaxKind::*;
        match node.kind {
            SourceFile | ModuleDeclaration | Block | CaseBlock | ForStatement | ForInStatement
            | ForOfStatement => out.push((Arc::clone(node), true)),
            Constructor | FunctionExpression | FunctionDeclaration | ArrowFunction
            | MethodDeclaration | GetAccessor | SetAccessor => {
                out.push((Arc::clone(node), Self::function_like_has_body(node)));
            }
            ClassDeclaration | ClassExpression | MethodSignature | CallSignature
            | ConstructSignature | FunctionType | ConstructorType | TypeAliasDeclaration
            | InterfaceDeclaration => out.push((Arc::clone(node), false)),
            _ => {}
        }
        tsox_frontend::ast::node_data_generated::for_each_child(node, |child| {
            Self::collect_unused_check_containers(child, out);
            false
        });
    }

    pub(crate) fn check_unused_class_members(&mut self, node: &Arc<Node>) {
        use tsox_core::diagnostics::messages_generated::{
            PROPERTY_0_IS_DECLARED_BUT_ITS_VALUE_IS_NEVER_READ,
            X_0_IS_DECLARED_BUT_NEVER_USED, X_0_IS_DECLARED_BUT_ITS_VALUE_IS_NEVER_READ,
        };
        let members: Vec<Arc<Node>> = match &node.data {
            tsox_frontend::ast::NodeData::ClassDeclaration(d) => {
                d.members.iter().cloned().collect()
            }
            tsox_frontend::ast::NodeData::ClassExpression(d) => {
                d.members.iter().cloned().collect()
            }
            _ => return,
        };
        let _ = &X_0_IS_DECLARED_BUT_NEVER_USED;
        for member in &members {
            match member.kind {
                SyntaxKind::MethodDeclaration
                | SyntaxKind::PropertyDeclaration
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor => {
                    let Some(sym) = self.program.symbol_map().symbol_of(member) else {
                        continue;
                    };
                    if member.kind == SyntaxKind::SetAccessor
                        && sym.flags.contains(SymbolFlags::GetAccessor)
                    {
                        continue;
                    }
                    let name_is_private = member
                        .name()
                        .is_some_and(|n| n.kind == SyntaxKind::PrivateIdentifier);
                    let referenced = self
                        .symbol_reference_kinds
                        .get(&sym.id())
                        .is_some_and(|k| !k.is_empty());
                    if !referenced
                        && (member.has_syntactic_modifier(ModifierFlags::Private) || name_is_private)
                        && !member
                            .flags
                            .contains(tsox_frontend::ast::NodeFlags::Ambient)
                    {
                        let name = sym.name.clone();
                        let loc = member.name().map(|n| n.loc).unwrap_or(member.loc);
                        self.report_unused(
                            member,
                            false,
                            loc,
                            &X_0_IS_DECLARED_BUT_ITS_VALUE_IS_NEVER_READ,
                            vec![name],
                        );
                    }
                }
                SyntaxKind::Constructor => {
                    let parameters: Vec<Arc<Node>> = match &member.data {
                        tsox_frontend::ast::NodeData::ConstructorDeclaration(d) => {
                            d.parameters.iter().cloned().collect()
                        }
                        _ => continue,
                    };
                    for parameter in &parameters {
                        let Some(sym) = self.program.symbol_map().symbol_of(parameter) else {
                            continue;
                        };
                        let referenced = self
                            .symbol_reference_kinds
                            .get(&sym.id())
                            .is_some_and(|k| !k.is_empty());
                        if !referenced
                            && parameter.has_syntactic_modifier(ModifierFlags::Private)
                        {
                            let name = sym.name.clone();
                            let loc = parameter
                                .name()
                                .map(|n| n.loc)
                                .unwrap_or(parameter.loc);
                            self.report_unused(
                                parameter,
                                false,
                                loc,
                                &PROPERTY_0_IS_DECLARED_BUT_ITS_VALUE_IS_NEVER_READ,
                                vec![name],
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub(crate) fn check_unused_type_parameters(&mut self, node: &Arc<Node>) {
        use tsox_core::diagnostics::messages_generated::{
            ALL_TYPE_PARAMETERS_ARE_UNUSED, X_0_IS_DECLARED_BUT_NEVER_USED,
        };
        let Some(list) = Self::type_parameter_list(node) else {
            return;
        };
        let params: Vec<Arc<Node>> = list.nodes.iter().cloned().collect();
        if params.is_empty() {
            return;
        }
        if params.len() > 1 && params.iter().all(|p| self.is_unreferenced_type_parameter(p)) {
            let loc = tsox_core::core::text::TextRange::new(list.loc.pos() - 1, list.loc.end() + 1);
            self.report_unused(node, true, loc, &ALL_TYPE_PARAMETERS_ARE_UNUSED, vec![]);
        } else {
            for p in &params {
                if self.is_unreferenced_type_parameter(p) {
                    let name = p.name().map(|n| n.text().to_string()).unwrap_or_default();
                    let loc = p.name().map(|n| n.loc).unwrap_or(p.loc);
                    self.report_unused(
                        p,
                        true,
                        loc,
                        &X_0_IS_DECLARED_BUT_NEVER_USED,
                        vec![name],
                    );
                }
            }
        }
    }

    pub(crate) fn is_unreferenced_type_parameter(&self, node: &Arc<Node>) -> bool {
        let underscore = node.name().is_some_and(|n| n.text().starts_with('_'));
        if underscore {
            return false;
        }
        let Some(sym) = self.program.symbol_map().symbol_of(node) else {
            return false;
        };
        if self.type_parameter_referenced(&sym) {
            return false;
        }
        let mut owner = node.parent();
        while let Some(o) = owner {
            if Self::type_parameter_list(&o).is_some() {
                if let Some(owner_sym) = self.program.symbol_map().symbol_of(&o) {
                    for decl in &owner_sym.declarations {
                        let Some(list) = Self::type_parameter_list(decl) else {
                            continue;
                        };
                        for p in &list.nodes {
                            let Some(p_sym) = self.program.symbol_map().symbol_of(p) else {
                                continue;
                            };
                            if p_sym.name == sym.name && self.type_parameter_referenced(&p_sym) {
                                return false;
                            }
                        }
                    }
                }
                break;
            }
            owner = o.parent();
        }
        true
    }

    fn type_parameter_referenced(&self, sym: &Arc<tsox_frontend::ast::Symbol>) -> bool {
        self.symbol_reference_kinds
            .get(&sym.id())
            .is_some_and(|k| k.intersects(SymbolFlags::TypeParameter))
    }

    pub(crate) fn type_parameter_list(
        node: &Arc<Node>,
    ) -> Option<Arc<tsox_frontend::ast::NodeList>> {
        use tsox_frontend::ast::NodeData::*;
        match &node.data {
            FunctionDeclaration(d) => d.type_parameters.clone(),
            ClassDeclaration(d) => d.type_parameters.clone(),
            ClassExpression(d) => d.type_parameters.clone(),
            InterfaceDeclaration(d) => d.type_parameters.clone(),
            TypeAliasDeclaration(d) => d.type_parameters.clone(),
            CallSignatureDeclaration(d) => d.type_parameters.clone(),
            ConstructSignatureDeclaration(d) => d.type_parameters.clone(),
            ConstructorDeclaration(d) => d.type_parameters.clone(),
            GetAccessorDeclaration(d) => d.type_parameters.clone(),
            SetAccessorDeclaration(d) => d.type_parameters.clone(),
            MethodSignatureDeclaration(d) => d.type_parameters.clone(),
            MethodDeclaration(d) => d.type_parameters.clone(),
            ArrowFunction(d) => d.type_parameters.clone(),
            FunctionExpression(d) => d.type_parameters.clone(),
            FunctionTypeNode(d) => d.type_parameters.clone(),
            ConstructorTypeNode(d) => d.type_parameters.clone(),
            _ => None,
        }
    }

    pub(crate) fn function_like_has_body(node: &Arc<Node>) -> bool {
        use tsox_frontend::ast::NodeData;
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
        let mut locals: Vec<Arc<tsox_frontend::ast::Symbol>> = self
            .program
            .symbol_map()
            .locals
            .get(&container.id())
            .map(|l| l.entries.values().cloned().collect())
            .unwrap_or_default();
        let file_is_module = self
            .current_file
            .as_ref()
            .is_some_and(|f| {
                f.external_module_indicator.is_some() || f.common_js_module_indicator.is_some()
            });
        if container.kind == SyntaxKind::SourceFile
            && file_is_module
            && let Some(file_sym) = self.program.symbol_map().symbols.get(&container.id())
        {
            let mut seen: std::collections::HashSet<u64> =
                locals.iter().map(|s| s.id()).collect();
            for sym in file_sym.members.entries.values() {
                if seen.insert(sym.id()) {
                    locals.push(Arc::clone(sym));
                }
            }
        }
        if locals.is_empty() {
            return;
        }

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
                            if let Some(parent) = root.parent().as_ref() {
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
                            && declaration.kind != SyntaxKind::FunctionExpression
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
                    cursor = cursor.parent().as_ref()?.clone();
                }
                SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern => {
                    cursor = cursor.parent().as_ref()?.clone();
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
            SyntaxKind::NamespaceImport => node.parent().unwrap_or_else(|| Arc::clone(node)),
            _ => node
                .parent()
                .clone()
                .and_then(|p| p.parent())
                .unwrap_or_else(|| Arc::clone(node)),
        }
    }

    pub(crate) fn report_unused_local(&mut self, node: &Arc<Node>, name: &str, is_type_decl: bool) {
        let message: &'static tsox_core::diagnostics::Message = if is_type_decl {
            &tsox_core::diagnostics::messages_generated::X_0_IS_DECLARED_BUT_NEVER_USED
        } else {
            &tsox_core::diagnostics::messages_generated::X_0_IS_DECLARED_BUT_ITS_VALUE_IS_NEVER_READ
        };
        let loc = Self::name_or_node_loc(node);
        let is_param = node.kind == SyntaxKind::Parameter;
        self.report_unused(node, is_param, loc, message, vec![name.to_string()]);
    }

    pub(crate) fn report_unused_variables(&mut self, list: &Arc<Node>) {
        let declarations: Vec<Arc<Node>> = match &list.data {
            tsox_frontend::ast::NodeData::VariableDeclarationList(d) => {
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
                &tsox_core::diagnostics::messages_generated::ALL_VARIABLES_ARE_UNUSED,
                vec![],
            );
        } else {
            self.report_unused_variable_declarations(&declarations);
        }
    }

    pub(crate) fn report_unused_parameters(&mut self, function: &Arc<Node>) {
        let parameters: Vec<Arc<Node>> = match &function.data {
            tsox_frontend::ast::NodeData::ConstructorDeclaration(d) => {
                d.parameters.iter().cloned().collect()
            }
            tsox_frontend::ast::NodeData::FunctionDeclaration(d) => {
                d.parameters.iter().cloned().collect()
            }
            tsox_frontend::ast::NodeData::FunctionExpression(d) => {
                d.parameters.iter().cloned().collect()
            }
            tsox_frontend::ast::NodeData::ArrowFunction(d) => {
                d.parameters.iter().cloned().collect()
            }
            tsox_frontend::ast::NodeData::MethodDeclaration(d) => {
                d.parameters.iter().cloned().collect()
            }
            tsox_frontend::ast::NodeData::GetAccessorDeclaration(d) => {
                d.parameters.iter().cloned().collect()
            }
            tsox_frontend::ast::NodeData::SetAccessorDeclaration(d) => {
                d.parameters.iter().cloned().collect()
            }
            _ => return,
        };
        self.report_unused_variable_declarations(&parameters);
    }

    pub(crate) fn report_unused_variable_declarations(&mut self, declarations: &[Arc<Node>]) {
        for declaration in declarations {
            let (name_node, is_pattern) = match &declaration.data {
                tsox_frontend::ast::NodeData::VariableDeclaration(d) => {
                    (Some(Arc::clone(&d.name)), Self::is_binding_pattern(&d.name))
                }
                tsox_frontend::ast::NodeData::ParameterDeclaration(d) => {
                    (Some(Arc::clone(&d.name)), Self::is_binding_pattern(&d.name))
                }
                tsox_frontend::ast::NodeData::BindingElement(d) => {
                    let n = d.name.clone();
                    let is_pattern = n.as_ref().is_some_and(|n| Self::is_binding_pattern(n));
                    (n, is_pattern)
                }
                _ => continue,
            };
            let Some(name_node) = name_node else { continue };

            if declaration.kind == SyntaxKind::Parameter {
                if let tsox_frontend::ast::NodeData::ParameterDeclaration(d) = &declaration.data {
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
                    &tsox_core::diagnostics::messages_generated::
                        X_0_IS_DECLARED_BUT_ITS_VALUE_IS_NEVER_READ,
                    vec![name],
                );
            }
        }
    }
}
