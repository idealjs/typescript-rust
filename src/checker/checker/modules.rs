use std::sync::Arc;

use crate::ast::{
    Node, NodeData, Symbol, SymbolFlags, SyntaxKind,
};


use super::*;

impl Checker {
    fn module_has_export_clause(&self, module_symbol: &Arc<Symbol>, name: &str) -> bool {
        use crate::ast::NodeData;
        let mut found = false;
        self.for_each_module_statement(module_symbol, |stmt| {
            if let NodeData::ExportDeclaration(d) = &stmt.data
                && let Some(clause) = &d.export_clause
                && let NodeData::NamedExports(ne) = &clause.data
            {
                for el in ne.elements.iter() {
                    if let NodeData::ExportSpecifier(spec) = &el.data
                        && spec.name.text().trim_matches(['"', '\'', '`']) == name
                    {
                        found = true;
                        return true;
                    }
                }
            }
            false
        });
        found
    }

    fn module_has_syntactic_default(&self, module_symbol: &Arc<Symbol>) -> bool {
        use crate::ast::NodeData;
        let mut found = false;
        self.for_each_module_statement(module_symbol, |stmt| {
            match &stmt.data {
                NodeData::ExportAssignment(d) if !d.is_export_equals => found = true,
                _ => {
                    if stmt.has_syntactic_modifier(crate::ast::ModifierFlags::Default) {
                        found = true;
                    }
                }
            }
            found
        });
        found
    }

    fn module_is_ambient_export_context(&self, module_symbol: &Arc<Symbol>) -> bool {
        use crate::ast::NodeData;
        let mut is_ambient = false;
        let mut has_export_declaration = false;
        for decl in &module_symbol.declarations {
            let ambient = match &decl.data {
                NodeData::ModuleDeclaration(_) => {
                    decl.has_syntactic_modifier(crate::ast::ModifierFlags::Ambient)
                        || self
                            .get_source_file_of_node(decl)
                            .is_some_and(|f| f.is_declaration_file)
                }
                NodeData::SourceFile(_) => self
                    .get_source_file_of_node(decl)
                    .is_some_and(|f| f.is_declaration_file),
                _ => false,
            };
            is_ambient |= ambient;
        }
        if !is_ambient {
            return false;
        }
        self.for_each_module_statement(module_symbol, |stmt| match &stmt.data {
            NodeData::ExportDeclaration(_) => {
                has_export_declaration = true;
                true
            }
            NodeData::ExportAssignment(_) => {
                has_export_declaration = true;
                true
            }
            _ => false,
        });
        !has_export_declaration
    }

    fn module_ambient_locals_contain(&self, module_symbol: &Arc<Symbol>, name: &str) -> bool {

        for decl in &module_symbol.declarations {
            if decl.kind == SyntaxKind::ModuleDeclaration
                && let Some(locals) = self.program.symbol_map().locals.get(&decl.id())
                && locals.get(name).is_some()
            {
                return true;
            }
        }
        false
    }

    fn module_star_chain_exports(&mut self, module_symbol: &Arc<Symbol>, name: &str) -> bool {
        if name == "default" {
            return false;
        }
        let stars = self.module_star_specs(module_symbol);
        let mut visited: Vec<*const Symbol> = vec![Arc::as_ptr(module_symbol)];
        for (spec, file) in &stars {
            if let Some(target) = self.resolve_module_symbol_from(spec, file)
                && self.star_target_exports(&target, name, &mut visited, 0)
            {
                return true;
            }
        }
        false
    }

    fn module_star_specs(
        &self,
        module_symbol: &Arc<Symbol>,
    ) -> Vec<(Arc<Node>, Arc<crate::ast::SourceFile>)> {
        use crate::ast::NodeData;
        let mut stars = Vec::new();
        self.for_each_module_statement(module_symbol, |stmt| {
            if let NodeData::ExportDeclaration(d) = &stmt.data
                && d.export_clause.is_none()
                && let Some(spec) = &d.module_specifier
                && let Some(file) = self.get_source_file_of_node(stmt)
            {
                stars.push((Arc::clone(spec), file));
            }
            false
        });
        stars
    }

    fn star_target_exports(
        &mut self,
        target: &Arc<Symbol>,
        name: &str,
        visited: &mut Vec<*const Symbol>,
        depth: usize,
    ) -> bool {
        if depth >= 8 || visited.contains(&Arc::as_ptr(target)) {
            return false;
        }
        visited.push(Arc::as_ptr(target));

        let face = match target.exports.get("export=") {
            Some(ee) => self.resolve_export_equals_target(ee),
            None => Arc::clone(target),
        };
        if face.exports.get(name).is_some()
            || self.module_has_export_clause(&face, name)
            || face
                .members
                .get(name)
                .is_some_and(|s| s.export_symbol.is_some())
            || (self.module_is_ambient_export_context(&face)
                && self.module_ambient_locals_contain(&face, name))
        {
            return true;
        }
        let stars = self.module_star_specs(&face);
        for (spec, file) in &stars {
            if let Some(next) = self.resolve_module_symbol_from(spec, file)
                && self.star_target_exports(&next, name, visited, depth + 1)
            {
                return true;
            }
        }
        false
    }

    fn resolve_module_symbol_from(
        &mut self,
        spec_node: &Arc<Node>,
        file: &Arc<crate::ast::SourceFile>,
    ) -> Option<Arc<Symbol>> {
        let spec_text = spec_node.text().trim_matches(['"', '\'', '`']).to_string();
        let file_symbol = |checker: &Self| {
            checker
                .program
                .resolve_external_module_path(
                    &spec_text,
                    &file.file_name,
                    crate::core::compiler_options::ModuleKind::None,
                )
                .and_then(|path| {
                    let sf = checker.program.get_source_file(&path)?;
                    checker.program.symbol_map().symbol_of(&sf.node).cloned()
                })
        };
        if !spec_text.starts_with('.') && !spec_text.starts_with("..") {
            self.resolve_module_file_symbol(&spec_text)
                .or_else(|| file_symbol(self))
        } else {
            file_symbol(self)
        }
    }

    fn resolve_export_equals_target(&mut self, export_equals: &Arc<Symbol>) -> Arc<Symbol> {
        let mut target = self.resolve_alias_base(Arc::clone(export_equals));
        for decl in export_equals.declarations.clone() {
            if let crate::ast::NodeData::ExportAssignment(d) = &decl.data
                && matches!(
                    d.expression.kind,
                    SyntaxKind::Identifier | SyntaxKind::QualifiedName
                )
            {
                if let Some(t) = self.with_declaring_file_context(&decl, |c| {
                    c.resolve_qualified_symbol(&d.expression)
                }) {

                    target = if t.flags.intersects(SymbolFlags::Alias) {
                        self.resolve_alias_base(t)
                    } else {
                        t
                    };
                }
                break;
            }
        }
        target
    }

    fn module_target_has_member(&self, target: &Arc<Symbol>, name: &str) -> bool {
        use crate::ast::NodeData;
        if target.exports.get(name).is_some() || target.members.get(name).is_some() {
            return true;
        }

        let mut has_export_declaration = false;
        let mut ambient = false;
        let mut locals_hit = false;
        for decl in &target.declarations {
            if decl.kind != SyntaxKind::ModuleDeclaration {
                continue;
            }
            if decl.has_syntactic_modifier(crate::ast::ModifierFlags::Ambient)
                || self
                    .get_source_file_of_node(decl)
                    .is_some_and(|f| f.is_declaration_file)
            {
                ambient = true;
            }
            let body = match &decl.data {
                NodeData::ModuleDeclaration(md) => md.body.clone(),
                _ => None,
            };
            if let Some(body) = body
                && let NodeData::ModuleBlock(b) = &body.data
            {
                for s in b.statements.iter() {
                    match &s.data {
                        NodeData::ExportDeclaration(_) | NodeData::ExportAssignment(_) => {
                            has_export_declaration = true;
                            break;
                        }
                        _ => {}
                    }
                }
            }
            if self
                .program
                .symbol_map()
                .locals
                .get(&decl.id())
                .is_some_and(|l| l.get(name).is_some())
            {
                locals_hit = true;
            }
        }
        ambient && !has_export_declaration && locals_hit
    }

    fn module_can_have_synthetic_default(&mut self, module_symbol: &Arc<Symbol>) -> bool {
        if self.module_has_syntactic_default(module_symbol) {
            return false;
        }
        if module_symbol.exports.get("__esModule").is_some() {
            return false;
        }
        let is_ambient_or_declaration = module_symbol.declarations.iter().any(|d| {
            match &d.data {
                crate::ast::NodeData::ModuleDeclaration(_) => true,
                crate::ast::NodeData::SourceFile(_) => self
                    .get_source_file_of_node(d)
                    .is_some_and(|f| f.is_declaration_file),
                _ => false,
            }
        });
        if is_ambient_or_declaration {
            return true;
        }
        module_symbol.exports.get("export=").is_some()
    }

    fn declaring_dir_of(&self, node: &Arc<Node>) -> Option<String> {
        self.get_source_file_of_node(node)
            .or_else(|| self.current_file.clone())
            .map(|f| match f.file_name.rfind('/') {
                Some(i) => f.file_name[..i].to_string(),
                None => String::new(),
            })
    }

    pub(crate) fn resolve_import_alias_module(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
        let decl = symbol
            .declarations
            .iter()
            .find(|d| {
                matches!(
                    d.kind,
                    SyntaxKind::NamespaceImport
                        | SyntaxKind::ImportSpecifier
                        | SyntaxKind::NamespaceExport
                )
            })?
            .clone();

        let mut cur = decl;
        for _ in 0..4 {
            let Some(parent) = cur.parent.clone() else {
                return None;
            };
            if parent.kind == SyntaxKind::ExportDeclaration {

                if let crate::ast::NodeData::ExportDeclaration(d) = &parent.data {
                    let Some(specifier) = &d.module_specifier else {
                        return None;
                    };
                    let spec = specifier.text();
                    if !spec.starts_with('.') {
                        return self.resolve_module_file_symbol(&spec);
                    }
                    let dir = self.declaring_dir_of(&parent)?;
                    return self.resolve_module_file_symbol_in(&dir, &spec);
                }
                return None;
            }
            if parent.kind == SyntaxKind::ImportDeclaration {
                if let crate::ast::NodeData::ImportDeclaration(d) = &parent.data {
                    let spec = d.module_specifier.text();
                    if !spec.starts_with('.') {
                        return self.resolve_module_file_symbol(&spec);
                    }

                    let dir = self
                        .get_source_file_of_node(&parent)
                        .map(|f| {
                            match f.file_name.rfind('/') {
                                Some(i) => f.file_name[..i].to_string(),
                                None => String::new(),
                            }
                        })
                        .or_else(|| {
                            self.current_file.as_ref().map(|f| {
                                match f.file_name.rfind('/') {
                                    Some(i) => f.file_name[..i].to_string(),
                                    None => String::new(),
                                }
                            })
                        })?;
                    return self.resolve_module_file_symbol_in(&dir, &spec);
                }
                return None;
            }
            cur = parent;
        }
        None
    }
    pub(crate) fn check_module_format_mismatch(&mut self, node: &Arc<Node>) {
        use crate::core::compiler_options::ModuleKind;
        if !matches!(self.module_kind, ModuleKind::Node16 | ModuleKind::Node18) {
            return;
        }
        let Some(file) = self.current_file.clone() else {
            return;
        };
        if file.file_name.starts_with("bundled://") {
            return;
        }
        let (spec_node, attrs, is_import_equals): (Arc<Node>, Option<Arc<Node>>, bool) =
            match &node.data {
                NodeData::ImportDeclaration(d) => (
                    Arc::clone(&d.module_specifier),
                    d.attributes.clone(),
                    false,
                ),
                NodeData::ExportDeclaration(d) => match &d.module_specifier {
                    Some(spec) => (Arc::clone(spec), d.attributes.clone(), false),
                    None => return,
                },
                NodeData::ImportEqualsDeclaration(d) => {
                    match &d.module_reference.data {
                        NodeData::ExternalModuleReference(ext) => {
                            (Arc::clone(&ext.expression), None, true)
                        }
                        _ => return,
                    }
                }
                _ => return,
            };

        if let Some(attrs) = &attrs
            && self.get_resolution_mode_override(attrs, false).is_some()
        {
            return;
        }
        let spec_text = spec_node.text().trim_matches(['"', '\'', '`']).to_string();
        if spec_text.is_empty() {
            return;
        }
        let read = |p: &str| self.program.read_file(p);
        let target_path = match self.program.resolve_external_module_path(
            &spec_text,
            &file.file_name,
            ModuleKind::None,
        ) {
            Some(p) => p,
            None => return,
        };

        if !module_format_is_esm_for_require_check(&target_path, &read) {
            return;
        }
        if is_import_equals {
            self.diagnostics.add(crate::ast::Diagnostic::new(
                Some(file),
                spec_node.loc,
                crate::diagnostics::messages_generated::
                    MODULE_0_CANNOT_BE_IMPORTED_USING_THIS_CONSTRUCT_THE_SPECIFIER_ONLY_RESOLVES_TO_AN_ES_MODULE_WHICH_CANNOT_BE_IMPORTED_WITH_REQUIRE_USE_AN_ECMASCRIPT_IMPORT_INSTEAD,
                vec![spec_text.clone()],
            ));
        } else if importer_is_cjs_for_require_check(&file.file_name, &read) {
            self.diagnostics.add(crate::ast::Diagnostic::new(
                Some(file),
                spec_node.loc,
                crate::diagnostics::messages_generated::
                    THE_CURRENT_FILE_IS_A_COMMONJS_MODULE_WHOSE_IMPORTS_WILL_PRODUCE_REQUIRE_CALLS_HOWEVER_THE_REFERENCED_FILE_IS_AN_ECMASCRIPT_MODULE_AND_CANNOT_BE_IMPORTED_WITH_REQUIRE_CONSIDER_WRITING_A_DYNAMIC_IMPORT_0_CALL_INSTEAD,
                vec![spec_text],
            ));
        }
    }

    pub(crate) fn check_declaration_nameability(&mut self, stmt: &Arc<Node>) {
        if !self.program.options().declaration.is_true() {
            return;
        }
        let Some(file) = self.current_file.clone() else {
            return;
        };
        if file.file_name.starts_with("bundled://") || file.is_declaration_file {
            return;
        }

        if file.file_name.contains("/node_modules/") {
            return;
        }
        let crate::ast::NodeData::VariableStatement(data) = &stmt.data else {
            return;
        };
        let has_export = stmt.has_syntactic_modifier(crate::ast::ModifierFlags::Export);
        if !has_export {
            return;
        }

        let mut imported_files: Vec<String> = Vec::new();
        let mut spec_names: Vec<String> = Vec::new();
        let NodeData::SourceFile(sfd) = &file.node.data else {
            return;
        };
        for st in sfd.statements.iter() {
            let spec = match &st.data {
                NodeData::ImportDeclaration(d) => d.module_specifier.text().to_string(),
                NodeData::ExportDeclaration(d) => match &d.module_specifier {
                    Some(s) => s.text().to_string(),
                    None => continue,
                },
                _ => continue,
            };
            let text = spec.trim_matches(['"', '\'', '`']).to_string();
            if text.is_empty() {
                continue;
            }
            spec_names.push(text.clone());
            if let Some(p) = self.program.resolve_external_module_path(
                &text,
                &file.file_name,
                crate::core::compiler_options::ModuleKind::None,
            ) {
                imported_files.push(p);
            }
        }
        let crate::ast::NodeData::VariableDeclarationList(list) = &data.declaration_list.data
        else {
            return;
        };
        for d in list.declarations.iter() {
            let crate::ast::NodeData::VariableDeclaration(vd) = &d.data else {
                continue;
            };

            if let Some(init) = &vd.initializer {
                let mut import_expr = Some(Arc::clone(init));
                if let Some(inner) = import_expr.take() {
                    let unwrapped = match &inner.data {
                        NodeData::AwaitExpression(a) => Some(Arc::clone(&a.expression)),
                        _ => Some(inner),
                    };
                    if let Some(call) = unwrapped
                        && call.kind == SyntaxKind::CallExpression
                        && let Some(spec) = self.spec_of_dynamic_import_call(&call)
                        && let Some(path) = self.program.resolve_external_module_path(
                            &spec,
                            &file.file_name,
                            crate::core::compiler_options::ModuleKind::ESNext,
                        )
                        && !imported_files.contains(&path)
                    {
                        imported_files.push(path);
                    }
                }
            }

            if vd.type_node.is_some() {
                continue;
            }
            let Some(sym) = self.program.symbol_map().symbol_of(d).cloned() else {
                continue;
            };
            let var_name = vd.name.text().to_string();
            let t = self.get_type_of_symbol(&sym);
            let Some(target) = t.symbol.clone() else {
                continue;
            };
            let Some(target_file) = target
                .declarations
                .first()
                .and_then(|dn| self.get_source_file_of_node(dn))
            else {
                continue;
            };
            if target_file.file_name == file.file_name
                || !target_file.file_name.contains("/node_modules/")
                || imported_files.contains(&target_file.file_name)
            {
                continue;
            }

            if self.symbol_in_ambient_module_named(&target, &spec_names) {
                continue;
            }

            let spec = relative_emit_specifier(&file.file_name, &target_file.file_name);
            self.diagnostics.add(crate::ast::Diagnostic::new(
                Some(file.clone()),
                vd.name.loc,
                crate::diagnostics::messages_generated::
                    THE_INFERRED_TYPE_OF_0_CANNOT_BE_NAMED_WITHOUT_A_REFERENCE_TO_2_FROM_1_THIS_IS_LIKELY_NOT_PORTABLE_A_TYPE_ANNOTATION_IS_NECESSARY,
                vec![var_name, spec, target.name.clone()],
            ));
        }
    }

    fn symbol_in_ambient_module_named(
        &self,
        symbol: &Arc<Symbol>,
        imported_specs: &[String],
    ) -> bool {
        if imported_specs.is_empty() {
            return false;
        }
        for decl in &symbol.declarations {
            let mut cur = decl.parent.as_ref();
            while let Some(n) = cur {
                if let NodeData::ModuleDeclaration(md) = &n.data
                    && md.name.kind == SyntaxKind::StringLiteral
                {
                    let module_name = md.name.text().trim_matches(['"', '\'']).to_string();
                    return imported_specs.iter().any(|s| *s == module_name);
                }
                if n.kind == SyntaxKind::SourceFile {
                    break;
                }
                cur = n.parent.as_ref();
            }
        }
        false
    }

    pub(crate) fn check_module_export_names(&mut self, node: &Arc<Node>) {
        use crate::core::compiler_options::ModuleKind;

        let mut names: Vec<(Arc<Node>, bool)> = Vec::new();
        match &node.data {
            NodeData::ImportDeclaration(d) => {
                let Some(clause) = &d.import_clause else { return };
                let NodeData::ImportClause(ic) = &clause.data else {
                    return;
                };
                let Some(named) = &ic.named_bindings else { return };
                let NodeData::NamedImports(ni) = &named.data else {
                    return;
                };
                for el in ni.elements.iter() {
                    if let NodeData::ImportSpecifier(spec) = &el.data {
                        if let Some(pn) = &spec.property_name {
                            names.push((Arc::clone(pn), true));
                        }
                    }
                }
            }
            NodeData::ExportDeclaration(d) => {
                let has_module_specifier = d.module_specifier.is_some();
                match &d.export_clause {
                    Some(clause) => match &clause.data {
                        NodeData::NamedExports(ne) => {
                            for el in ne.elements.iter() {
                                if let NodeData::ExportSpecifier(spec) = &el.data {
                                    if let Some(pn) = &spec.property_name {
                                        names.push((Arc::clone(pn), has_module_specifier));
                                    }
                                    names.push((Arc::clone(&spec.name), true));
                                }
                            }
                        }
                        NodeData::NamespaceExport(ne) => {
                            names.push((Arc::clone(&ne.name), true));
                        }
                        _ => {}
                    },
                    None => {}
                }
            }
            _ => return,
        }
        if names.is_empty() {
            return;
        }
        let declaration_file = self
            .current_file
            .as_ref()
            .is_some_and(|f| f.is_declaration_file);
        for (name, string_allowed) in names {
            if name.kind != SyntaxKind::StringLiteral {
                continue;
            }
            if !string_allowed {
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    name.loc,
                    crate::diagnostics::messages_generated::IDENTIFIER_EXPECTED,
                    vec![],
                ));
            } else if matches!(self.module_kind, ModuleKind::ES2015 | ModuleKind::ES2020)
                && !declaration_file
            {
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    name.loc,
                    crate::diagnostics::messages_generated::
                        STRING_LITERAL_IMPORT_AND_EXPORT_NAMES_ARE_NOT_SUPPORTED_WHEN_THE_MODULE_FLAG_IS_SET_TO_ES2015_OR_ES2020,
                    vec![],
                ));
            }
        }
    }

    pub(crate) fn check_module_specifier_members(&mut self, node: &Arc<Node>) {
        use crate::ast::NodeData;

        let (spec_node, attrs, exclusively_type_only, elements): (
            Arc<Node>,
            Option<Arc<Node>>,
            bool,
            Arc<crate::ast::NodeList>,
        ) = match &node.data {
            NodeData::ImportDeclaration(d) => {
                let Some(clause) = &d.import_clause else { return };
                let NodeData::ImportClause(ic) = &clause.data else {
                    return;
                };
                let Some(named) = &ic.named_bindings else { return };
                let NodeData::NamedImports(ni) = &named.data else {
                    return;
                };
                (
                    Arc::clone(&d.module_specifier),
                    d.attributes.clone(),
                    ic.phase_modifier == Some(SyntaxKind::TypeKeyword),
                    Arc::clone(&ni.elements),
                )
            }
            NodeData::ExportDeclaration(d) => {
                let Some(spec) = &d.module_specifier else {
                    return;
                };
                let Some(clause) = &d.export_clause else {
                    return;
                };
                let NodeData::NamedExports(ne) = &clause.data else {
                    return;
                };
                (
                    Arc::clone(spec),
                    d.attributes.clone(),
                    d.is_type_only,
                    Arc::clone(&ne.elements),
                )
            }
            _ => return,
        };
        if elements.is_empty() {
            return;
        }
        let Some(file) = self.current_file.clone() else {
            return;
        };
        let spec_text = spec_node.text().trim_matches(['"', '\'', '`']).to_string();

        let mode = match (&attrs, exclusively_type_only) {
            (Some(attrs), true) => self
                .get_resolution_mode_override(attrs, false)
                .unwrap_or(crate::core::compiler_options::ModuleKind::None),
            _ => crate::core::compiler_options::ModuleKind::None,
        };

        let file_symbol = |checker: &Self| {
            checker
                .program
                .resolve_external_module_path(&spec_text, &file.file_name, mode)
                .and_then(|path| {
                    let sf = checker.program.get_source_file(&path)?;
                    checker.program.symbol_map().symbol_of(&sf.node).cloned()
                })
        };
        let module_symbol = if !spec_text.starts_with('.') && !spec_text.starts_with("..") {
            self.resolve_module_file_symbol(&spec_text)
                .or_else(|| file_symbol(self))
        } else {
            file_symbol(self)
        };
        let Some(module_symbol) = module_symbol else {
            return;
        };

        let shorthand_ambient = module_symbol.value_declaration.as_ref().is_some_and(|d| {
            matches!(&d.data, NodeData::ModuleDeclaration(md) if md.body.is_none())
        });
        if shorthand_ambient {
            return;
        }
        for element in elements.iter() {
            let (property_name, name) = match &element.data {
                NodeData::ImportSpecifier(d) => (d.property_name.clone(), d.name.clone()),
                NodeData::ExportSpecifier(d) => (d.property_name.clone(), d.name.clone()),
                _ => continue,
            };
            let member_name = property_name
                .as_ref()
                .unwrap_or(&name)
                .text()
                .trim_matches(['"', '\'', '`'])
                .to_string();
            let error_node = property_name.clone().unwrap_or_else(|| Arc::clone(&name));
            match self.module_member_lookup(&module_symbol, &member_name) {
                ModuleMemberLookup::Found => {}

                ModuleMemberLookup::LocalNotExported => {
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        Some(file.clone()),
                        error_node.loc,
                        crate::diagnostics::messages_generated::
                            MODULE_0_DECLARES_1_LOCALLY_BUT_IT_IS_NOT_EXPORTED,
                        vec![format!("\"{spec_text}\""), member_name],
                    ));
                }
                ModuleMemberLookup::Missing => {
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        Some(file.clone()),
                        error_node.loc,
                        crate::diagnostics::messages_generated::MODULE_0_HAS_NO_EXPORTED_MEMBER_1,

                        vec![format!("\"{spec_text}\""), member_name],
                    ));
                }
            }
        }
    }

    fn module_member_lookup(
        &mut self,
        module_symbol: &Arc<Symbol>,
        name: &str,
    ) -> ModuleMemberLookup {
        use ModuleMemberLookup as M;

        if let Some(export_equals) = module_symbol.exports.get("export=") {
            let target = self.resolve_export_equals_target(export_equals);
            if std::env::var_os("TSOX_DEBUG_MODULE").is_some() {
                eprintln!(
                    "[mod-lookup] export= chain: module={:?} target={:?} exports={} members={}",
                    module_symbol.name,
                    target.name,
                    target.exports.len(),
                    target.members.len()
                );
            }
            if self.module_target_has_member(&target, name)
                || module_symbol.exports.get(name).is_some()
            {
                return M::Found;
            }

            if self.module_star_chain_exports(module_symbol, name)
                || (name == "default"
                    && self.module_can_have_synthetic_default(module_symbol))
            {
                return M::Found;
            }
            return M::Missing;
        }
        if module_symbol.exports.get(name).is_some() {
            return M::Found;
        }
        if std::env::var_os("TSOX_DEBUG_MODULE").is_some() {
            eprintln!(
                "[mod-lookup] plain: name={name} exports={:?} members_with={:?} decls={:?}",
                module_symbol.exports.iter().take(12).map(|(k, _)| k.clone()).collect::<Vec<_>>(),
                module_symbol
                    .members
                    .get(name)
                    .map(|s| (s.export_symbol.is_some(), s.flags)),
                module_symbol.declarations.iter().map(|d| d.kind).collect::<Vec<_>>()
            );
        }

        if self.module_has_export_clause(module_symbol, name) {
            return M::Found;
        }

        if name == "default" && self.module_has_syntactic_default(module_symbol) {
            return M::Found;
        }
        if let Some(sym) = module_symbol.members.get(name) {
            return if sym.export_symbol.is_some() {
                M::Found
            } else {
                M::LocalNotExported
            };
        }

        if let Some(file_node) = module_symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::SourceFile)
        {
            if let Some(locals) = self.program.symbol_map().locals.get(&file_node.id())
                && let Some(sym) = locals.get(name)
            {
                return if sym.export_symbol.is_some() {
                    M::Found
                } else {
                    M::LocalNotExported
                };
            }
        }

        if self.module_is_ambient_export_context(module_symbol)
            && self.module_ambient_locals_contain(module_symbol, name)
        {
            return M::Found;
        }

        if name != "default" && self.module_star_chain_exports(module_symbol, name) {
            return M::Found;
        }

        if name == "default" && self.module_can_have_synthetic_default(module_symbol) {
            return M::Found;
        }
        M::Missing
    }
}
