#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn check_module_declaration(&mut self, node: &Arc<Node>) {
            self.check_grammar_modifiers(node);

            if let crate::ast::NodeData::ModuleDeclaration(data) = &node.data
                && data.name.kind == SyntaxKind::Identifier
                && !is_valid_identifier_text(data.name.text())
            {
                if let Some(msg) = Self::cannot_find_name_message_for("module") {
                    let file = self.current_file.clone();
                    let kw = crate::core::text::TextRange::new(
                        node.loc.pos(),
                        (node.loc.pos() + 6).min(node.loc.end()),
                    );
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        kw,
                        *msg,
                        vec!["module".to_string()],
                    ));
                }
            }

            if let crate::ast::NodeData::ModuleDeclaration(data) = &node.data
                && data.name.kind == SyntaxKind::StringLiteral
                && self
                    .current_file
                    .as_ref()
                    .is_some_and(|f| !f.file_name.starts_with("bundled://"))
            {
                let raw = data.name.text();
                let module_name = raw.trim_matches(['"', '\'']);
                let relative = module_name.starts_with("./")
                    || module_name.starts_with("../")
                    || module_name.starts_with(".\\")
                    || module_name.starts_with("..\\");
                let ambient = node.has_syntactic_modifier(ModifierFlags::Ambient)
                    || self.ambient_context_depth > 0
                    || self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| f.is_declaration_file);

                if relative && ambient {
                    let is_decl_name_direct = !self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| f.external_module_indicator.is_some());
                    if is_decl_name_direct {
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            data.name.loc,
                            crate::diagnostics::messages_generated::
                                AMBIENT_MODULE_DECLARATION_CANNOT_SPECIFY_RELATIVE_MODULE_NAME,
                            vec![],
                        ));
                    }
                }
            }

            if let crate::ast::NodeData::ModuleDeclaration(data) = &node.data
                && data.name.kind == SyntaxKind::StringLiteral
                && self.current_file.as_ref().is_some_and(|f| {
                    f.external_module_indicator.is_some()
                        && !f.file_name.starts_with("bundled://")
                })
            {
                let module_name = data.name.text().trim_matches(['"', '\'']).to_string();
                let resolvable = self.resolve_module_file_symbol(&module_name).is_some();
                if !resolvable {
                    let file = self.current_file.clone();
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        data.name.loc,
                        crate::diagnostics::messages_generated::
                            INVALID_MODULE_NAME_IN_AUGMENTATION_MODULE_0_CANNOT_BE_FOUND,
                        vec![module_name],
                    ));
                }
            }

            if let crate::ast::NodeData::ModuleDeclaration(mdd) = &node.data
                && mdd.name.kind == SyntaxKind::Identifier
                && !node.has_syntactic_modifier(ModifierFlags::Ambient)
                && self.ambient_context_depth == 0
                && !self
                    .current_file
                    .as_ref()
                    .is_some_and(|f| f.is_declaration_file)
                && let Some(sym) = self.program.symbol_map().symbol_of(node)
            {
                if sym.flags.contains(SymbolFlags::ValueModule)
                    && sym.declarations.len() > 1
                    && module_is_instantiated(
                        node,
                        self.compiler_options.should_preserve_const_enums(),
                    )
                {
                    let first_non_ambient = sym.declarations.iter().find(|d| {
                        let bodied_fn = matches!(
                            &d.data,
                            crate::ast::NodeData::FunctionDeclaration(fd)
                                if fd.body.is_some()
                        );
                        (matches!(d.kind, SyntaxKind::ClassDeclaration) || bodied_fn)
                            && !d.has_syntactic_modifier(ModifierFlags::Ambient)
                            && !self
                                .get_source_file_of_node(d)
                                .is_some_and(|f| f.is_declaration_file)
                    });
                    if let Some(fc) = first_non_ambient
                        && node.loc.pos() < fc.loc.pos()
                    {
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                        self.current_file.clone(),
                        mdd.name.loc,
                        crate::diagnostics::messages_generated::
                            A_NAMESPACE_DECLARATION_CANNOT_BE_LOCATED_PRIOR_TO_A_CLASS_OR_FUNCTION_WITH_WHICH_IT_IS_MERGED,
                        Vec::new(),
                    ));
                    }
                }
            }

            let is_ambient = node.has_syntactic_modifier(ModifierFlags::Ambient);
            if is_ambient {
                self.ambient_context_depth += 1;
            }
            self.push_scope(node);
            if let crate::ast::NodeData::ModuleDeclaration(data) = &node.data {
                if let Some(body) = &data.body {
                    self.check_statement(body);
                }
            }
            self.pop_scope();
            if is_ambient {
                self.ambient_context_depth -= 1;
            }
    }
}