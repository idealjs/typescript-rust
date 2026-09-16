#![allow(unused_imports)]

use crate::checker::checker_statements::*;

impl Checker {
    pub fn check_module_declaration(&mut self, node: &Arc<Node>) {
        self.check_grammar_modifiers(node);

        // Go checkModuleDeclaration：global 增强诊断
        if tsox_frontend::ast::is_global_scope_augmentation(node) {
            let in_ambient = node.has_syntactic_modifier(ModifierFlags::Ambient)
                || self.ambient_context_depth > 0
                || self
                    .current_file
                    .as_ref()
                    .is_some_and(|f| f.is_declaration_file);
            let name_loc = match &node.data {
                tsox_frontend::ast::NodeData::ModuleDeclaration(d) => d.name.loc,
                _ => node.loc,
            };
            if !in_ambient {
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    name_loc,
                    tsox_core::diagnostics::messages_generated::
                        AUGMENTATIONS_FOR_THE_GLOBAL_SCOPE_SHOULD_HAVE_DECLARE_MODIFIER_UNLESS_THEY_APPEAR_IN_ALREADY_AMBIENT_CONTEXT,
                    Vec::new(),
                ));
            }
            let file_is_external = self
                .current_file
                .as_ref()
                .is_some_and(|f| f.external_module_indicator.is_some());
            // Go IsModuleAugmentationExternal：顶层时文件须为外部模块；
            // 位于模块块内时，祖父须为顶层 ambient 模块且该文件非外部模块
            let augmentation_external = match node.parent() {
                Some(p) if p.kind == SyntaxKind::SourceFile => file_is_external,
                Some(p) if p.kind == SyntaxKind::ModuleBlock => p.parent().is_some_and(|g| {
                    g.kind == SyntaxKind::ModuleDeclaration
                        && tsox_frontend::ast::is_ambient_module(&g)
                        && g.parent()
                            .is_some_and(|gg| gg.kind == SyntaxKind::SourceFile)
                        && !file_is_external
                }),
                _ => false,
            };
            if !augmentation_external {
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    name_loc,
                    tsox_core::diagnostics::messages_generated::
                        AUGMENTATIONS_FOR_THE_GLOBAL_SCOPE_CAN_ONLY_BE_DIRECTLY_NESTED_IN_EXTERNAL_MODULES_OR_AMBIENT_MODULE_DECLARATIONS,
                    Vec::new(),
                ));
            }
        }

        if let tsox_frontend::ast::NodeData::ModuleDeclaration(data) = &node.data
            && data.name.kind == SyntaxKind::Identifier
            && !is_valid_identifier_text(data.name.text())
        {
            if let Some(msg) = Self::cannot_find_name_message_for("module", None) {
                let file = self.current_file.clone();
                let kw = tsox_core::core::text::TextRange::new(
                    node.loc.pos(),
                    (node.loc.pos() + 6).min(node.loc.end()),
                );
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    kw,
                    *msg,
                    vec!["module".to_string()],
                ));
            }
        }

        if let tsox_frontend::ast::NodeData::ModuleDeclaration(data) = &node.data
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
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            file,
                            data.name.loc,
                            tsox_core::diagnostics::messages_generated::
                                AMBIENT_MODULE_DECLARATION_CANNOT_SPECIFY_RELATIVE_MODULE_NAME,
                            vec![],
                        ));
                }
            }
        }

        if let tsox_frontend::ast::NodeData::ModuleDeclaration(data) = &node.data
            && data.name.kind == SyntaxKind::StringLiteral
            && self.current_file.as_ref().is_some_and(|f| {
                f.external_module_indicator.is_some() && !f.file_name.starts_with("bundled://")
            })
        {
            let module_name = data.name.text().trim_matches(['"', '\'']).to_string();
            let resolvable = self.resolve_module_file_symbol(&module_name).is_some();
            if !resolvable {
                let file = self.current_file.clone();
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        file,
                        data.name.loc,
                        tsox_core::diagnostics::messages_generated::
                            INVALID_MODULE_NAME_IN_AUGMENTATION_MODULE_0_CANNOT_BE_FOUND,
                        vec![module_name],
                    ));
            }
        }

        if let tsox_frontend::ast::NodeData::ModuleDeclaration(mdd) = &node.data
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
                && module_is_instantiated(node, self.compiler_options.should_preserve_const_enums())
            {
                let first_non_ambient = sym.declarations.iter().find(|d| {
                    let bodied_fn = matches!(
                        &d.data,
                        tsox_frontend::ast::NodeData::FunctionDeclaration(fd)
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
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        self.current_file.clone(),
                        mdd.name.loc,
                        tsox_core::diagnostics::messages_generated::
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
        if let tsox_frontend::ast::NodeData::ModuleDeclaration(data) = &node.data {
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
