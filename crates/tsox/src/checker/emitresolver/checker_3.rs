#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn is_declaration_visible(&mut self, node: &Arc<Node>) -> bool {
        let cached = self
            .declaration_links
            .get(node)
            .map(|l| l.is_visible)
            .unwrap_or_default();
        if !cached.is_unknown() {
            return cached.is_true();
        }
        let result = self.determine_if_declaration_is_visible(node);
        self.declaration_links.get_or_default(node).is_visible = result.into();
        result
    }

    pub(crate) fn determine_if_declaration_is_visible(&mut self, node: &Arc<Node>) -> bool {
        match node.kind {
            SyntaxKind::BindingElement => node
                .parent
                .clone()
                .and_then(|p| p.parent.clone())
                .map(|gp| self.is_declaration_visible(&gp))
                .unwrap_or(false),

            SyntaxKind::VariableDeclaration
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::ImportEqualsDeclaration => {
                if node.kind == SyntaxKind::VariableDeclaration {
                    if let NodeData::VariableDeclaration(d) = &node.data {
                        let name = &d.name;
                        if name.kind == SyntaxKind::ObjectBindingPattern
                            || name.kind == SyntaxKind::ArrayBindingPattern
                        {
                            if let NodeData::BindingPattern(p) = &name.data {
                                if p.elements.nodes.is_empty() {
                                    return false;
                                }
                            }
                        }
                    }
                }

                if Self::is_external_module_augmentation(node) {
                    return true;
                }
                let parent = match Checker::get_declaration_container(node) {
                    Some(p) => p,
                    None => return false,
                };
                let is_exported = self
                    .get_combined_modifier_flags(node)
                    .contains(crate::ast::ModifierFlags::Export);

                let is_ambient_element = node.kind != SyntaxKind::ImportEqualsDeclaration
                    && parent.kind != SyntaxKind::SourceFile
                    && parent.flags.contains(crate::ast::NodeFlags::Ambient);
                if !is_exported && !is_ambient_element {
                    return Self::is_global_source_file(&parent);
                }

                self.is_declaration_visible(&parent)
            }

            SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature => {
                let flags = self.get_effective_declaration_flags(node);
                let private_protected = crate::ast::ModifierFlags::Private
                    .union(crate::ast::ModifierFlags::Protected)
                    .bits();
                if flags & private_protected != 0 {
                    return false;
                }
                node.parent
                    .clone()
                    .map(|p| self.is_declaration_visible(&p))
                    .unwrap_or(false)
            }

            SyntaxKind::Constructor
            | SyntaxKind::ConstructSignature
            | SyntaxKind::CallSignature
            | SyntaxKind::IndexSignature
            | SyntaxKind::Parameter
            | SyntaxKind::ModuleBlock
            | SyntaxKind::FunctionType
            | SyntaxKind::ConstructorType
            | SyntaxKind::TypeLiteral
            | SyntaxKind::TypeReference
            | SyntaxKind::ArrayType
            | SyntaxKind::TupleType
            | SyntaxKind::UnionType
            | SyntaxKind::IntersectionType
            | SyntaxKind::ParenthesizedType
            | SyntaxKind::NamedTupleMember => node
                .parent
                .clone()
                .map(|p| self.is_declaration_visible(&p))
                .unwrap_or(false),

            SyntaxKind::ImportClause
            | SyntaxKind::NamespaceImport
            | SyntaxKind::ImportSpecifier => false,

            SyntaxKind::TypeParameter => true,

            SyntaxKind::SourceFile | SyntaxKind::NamespaceExportDeclaration => true,

            SyntaxKind::ExportAssignment => false,

            SyntaxKind::ExportSpecifier => {
                let export_decl = match node.parent.clone().and_then(|p| p.parent.clone()) {
                    Some(ed) if ed.kind == SyntaxKind::ExportDeclaration => ed,
                    _ => return false,
                };
                let has_module_specifier = match &export_decl.data {
                    NodeData::ExportDeclaration(d) => d.module_specifier.is_some(),
                    _ => false,
                };
                if has_module_specifier {
                    return false;
                }
                export_decl
                    .parent
                    .clone()
                    .map(|p| self.is_declaration_visible(&p))
                    .unwrap_or(false)
            }

            _ => false,
        }
    }

    pub fn precalculate_declaration_emit_visibility(&mut self, file: &Arc<crate::ast::SourceFile>) {
        if self
            .declaration_file_links
            .get(file)
            .map(|l| l.aliases_marked)
            .unwrap_or(false)
        {
            return;
        }
        self.declaration_file_links
            .get_or_default(file)
            .aliases_marked = true;

        let saved_file = self.current_file.take();
        let saved_file_id = self.current_file_id;
        let saved_file_symbol = self.current_file_symbol.take();
        let saved_scope_stack = self.scope_stack.clone();

        self.current_file = Some(Arc::clone(file));
        self.current_file_id = file.node.id();
        self.current_file_symbol = self.program.symbol_map().symbol_of(&file.node).cloned();

        self.scope_stack.clear();
        self.scope_stack.push(file.node.id());

        let children = collect_children(&file.node);
        for child in children {
            self.alias_marking_visitor(&child);
        }

        self.current_file = saved_file;
        self.current_file_id = saved_file_id;
        self.current_file_symbol = saved_file_symbol;
        self.scope_stack = saved_scope_stack;
    }

    pub(crate) fn alias_marking_visitor(&mut self, node: &Arc<Node>) {
        match node.kind {
            SyntaxKind::BinaryExpression => {
                if Self::is_common_js_module_exports(node) {
                    if let NodeData::BinaryExpression(bin) = &node.data {
                        if bin.right.kind == SyntaxKind::Identifier {
                            self.mark_linked_aliases(&bin.right);
                        }
                    }
                }
            }
            SyntaxKind::ExportAssignment => {
                if let Some(expr) = node.expression() {
                    if expr.kind == SyntaxKind::Identifier {
                        self.mark_linked_aliases(expr);
                    }
                }
            }
            SyntaxKind::ExportSpecifier => {
                if let Some(name) = Self::export_specifier_name(node) {
                    self.mark_linked_aliases(&name);
                }
            }
            _ => {}
        }

        let children = collect_children(node);
        for child in children {
            self.alias_marking_visitor(&child);
        }
    }

    pub(crate) fn mark_linked_aliases(&mut self, node: &Arc<Node>) {
        let export_symbol = self.resolve_export_symbol_for_alias(node);
        let mut export_symbol = export_symbol;

        let mut visited: Vec<u64> = Vec::new();
        while let Some(sym) = export_symbol {
            if visited.contains(&sym.id()) {
                break;
            }
            visited.push(sym.id());

            let mut next_symbol: Option<Arc<Symbol>> = None;
            let declarations = sym.declarations.clone();
            for declaration in declarations.iter() {
                self.declaration_links
                    .get_or_default(declaration)
                    .is_visible = true.into();

                if declaration.kind == SyntaxKind::ImportEqualsDeclaration {
                    if let NodeData::ImportEqualsDeclaration(d) = &declaration.data {
                        let first_id = Self::first_identifier_of(&d.module_reference);
                        if let Some(first_id) = first_id {
                            let saved = self.scope_stack.clone();

                            if let Some(parent) = declaration.parent.clone() {
                                self.scope_stack.push(parent.id());
                            }
                            let resolved = self.resolve_identifier_with_meaning(
                                &first_id,
                                SymbolFlags::VALUE
                                    | SymbolFlags::TYPE
                                    | SymbolFlags::NAMESPACE
                                    | SymbolFlags::Alias,
                            );
                            self.scope_stack = saved;
                            next_symbol = resolved;
                        }
                    }
                }
            }
            export_symbol = next_symbol;
        }
    }

    pub(crate) fn resolve_export_symbol_for_alias(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        let parent = node.parent.clone()?;
        match parent.kind {
            SyntaxKind::ExportAssignment | SyntaxKind::BinaryExpression => {
                let name = node.text();
                self.resolve_name_in_file_scope(name)
            }
            SyntaxKind::ExportSpecifier => {
                let spec_name = Self::export_specifier_name(&parent)?;
                let name = spec_name.text();
                self.resolve_name_in_file_scope(name)
            }
            _ => None,
        }
    }

}
