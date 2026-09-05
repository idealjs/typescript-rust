use std::sync::Arc;

use crate::ast::{
    Node, NodeData, NodeList, Symbol, SymbolFlags, SyntaxKind,
};







use super::*;


impl Checker {
    pub(crate) fn enclosing_function_is_generator(&self, node: &Arc<Node>) -> bool {
        let mut cur = node.parent.clone();
        while let Some(n) = cur {

            let in_name_of_current = crate::ast::node_data_generated::node_name(&n).is_some_and(
                |name| {
                    name.loc.pos() <= node.loc.pos() && node.loc.end() <= name.loc.end()
                },
            );
            if in_name_of_current {
                cur = n.parent.clone();
                continue;
            }
            match &n.data {
                crate::ast::NodeData::FunctionDeclaration(d) => {
                    return d.asterisk_token.is_some();
                }
                crate::ast::NodeData::FunctionExpression(d) => {
                    return d.asterisk_token.is_some();
                }
                crate::ast::NodeData::MethodDeclaration(d) => {
                    return d.asterisk_token.is_some();
                }

                crate::ast::NodeData::ArrowFunction(_)
                | crate::ast::NodeData::GetAccessorDeclaration(_)
                | crate::ast::NodeData::SetAccessorDeclaration(_)
                | crate::ast::NodeData::ConstructorDeclaration(_) => return false,
                _ => {}
            }
            cur = n.parent.clone();
        }
        false
    }


    pub(crate) fn get_array_element_type(&self, t: &Arc<Type>) -> Arc<Type> {
        match &t.data {
            crate::checker::TypeData::Object(obj) => {

                if let Some(elem) = obj.type_arguments.first() {
                    return Arc::clone(elem);
                }
                self.get_any_type()
            }
            crate::checker::TypeData::EvolvingArray(ea) => ea
                .element_type
                .clone()
                .unwrap_or_else(|| self.get_any_type()),
            _ => self.get_any_type(),
        }
    }


    pub(crate) fn is_empty_array_literal(&self, node: &Arc<Node>) -> bool {
        matches!(
            &node.data,
            crate::ast::NodeData::ArrayLiteralExpression(d) if d.elements.is_empty()
        )
    }


    pub(crate) fn get_missing_required_properties(
        &self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> Vec<String> {
        let Some(source_struct) = source.as_structured() else {
            return Vec::new();
        };
        let Some(target_struct) = target.as_structured() else {
            return Vec::new();
        };
        let mut missing = Vec::new();
        for target_prop in &target_struct.properties {
            if target_prop.flags.contains(SymbolFlags::Optional) {
                continue;
            }
            if source_struct.members.get(&target_prop.name).is_none() {
                missing.push(target_prop.name.clone());
            }
        }
        missing
    }


    pub(crate) fn get_property_name_from_node(&self, node: &Arc<Node>) -> String {
        match &node.data {
            NodeData::Identifier(id) => id.text.clone(),
            NodeData::StringLiteral(s) => s.text.clone(),
            NodeData::NumericLiteral(n) => n.text.clone(),
            NodeData::ComputedPropertyName(_) => {

                let file = self
                    .get_source_file_of_node(node)
                    .or_else(|| self.current_file.clone());
                let Some(file) = file else {
                    return String::new();
                };
                let pos = node.loc.pos();
                let end = node.loc.end();
                if pos < end && end <= file.text.len() {
                    file.text[pos..end].to_string()
                } else {
                    String::new()
                }
            }
            _ => node.text().to_string(),
        }
    }




    pub(crate) fn build_class_instance_type_with_base(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (members, heritage_clauses) = match &node.data {
            crate::ast::NodeData::ClassDeclaration(data) => {
                (&data.members, data.heritage_clauses.clone())
            }

            crate::ast::NodeData::ClassExpression(data) => {
                (&data.members, data.heritage_clauses.clone())
            }
            _ => return self.build_interface_type_from_members(&Arc::new(NodeList::default())),
        };

        let own_type = self.build_interface_type_from_members(members);

        if let Some(class_sym) = self.program.symbol_map().symbol_of(node) {
            let own_mut = Arc::as_ptr(&own_type) as *mut crate::checker::types::Type;
            unsafe {
                (*own_mut).symbol = Some(Arc::clone(class_sym));
            }
        }

        let mut base_type: Option<Arc<Type>> = None;
        if let Some(ref heritage) = heritage_clauses {
            for clause in heritage.iter() {
                if let crate::ast::NodeData::HeritageClause(hc) = &clause.data {
                    if hc.token == SyntaxKind::ExtendsKeyword {
                        if let Some(type_ref) = hc.types.iter().next() {
                            base_type = Some(self.resolve_base_class_instance_type(type_ref));
                        }
                        break;
                    }
                }
            }
        }
        match base_type {
            Some(base) => self.merge_instance_types(&own_type, &base),
            None => own_type,
        }
    }


    pub(crate) fn get_constituent_property(
        &mut self,
        object_type: &Arc<Type>,
        name: &str,
    ) -> Option<std::sync::Arc<crate::ast::Symbol>> {
        let apparent = self.get_apparent_type(object_type);
        let parts: Vec<Arc<Type>> = if apparent.flags.contains(
            crate::checker::types::TypeFlags::Union,
        ) {
            match &apparent.data {
                crate::checker::types::TypeData::Union(u) => u.union_or_intersection.types.clone(),
                _ => vec![apparent],
            }
        } else {
            vec![apparent]
        };
        for p in parts {
            if let Some(sym) = self.get_property_of_type(&p, name) {
                return Some(sym);
            }
        }
        None
    }

    pub(crate) fn loop_has_escaping_break(n: &Arc<Node>, direct: bool) -> bool {
        match n.kind {
            SyntaxKind::BreakStatement => {
                matches!(
                    &n.data,
                    crate::ast::NodeData::BreakStatement(d) if d.label.is_some()
                ) || direct
            }
            SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::Constructor => false,
            _ => {

                let nested = matches!(
                    n.kind,
                    SyntaxKind::WhileStatement
                        | SyntaxKind::DoStatement
                        | SyntaxKind::ForStatement
                        | SyntaxKind::ForInStatement
                        | SyntaxKind::ForOfStatement
                        | SyntaxKind::SwitchStatement
                );
                let mut found = false;
                crate::ast::node_data_generated::for_each_child(n, |child| {
                    if Self::loop_has_escaping_break(child, direct && !nested) {
                        found = true;
                        true
                    } else {
                        false
                    }
                });
                found
            }
        }
    }

    pub(crate) fn function_body_has_explicit_return(body: &Arc<Node>) -> bool {
        fn walk(n: &Arc<Node>) -> bool {
            match n.kind {
                SyntaxKind::ReturnStatement => return true,

                SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionExpression
                | SyntaxKind::ArrowFunction
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor => return false,
                _ => {}
            }
            let mut found = false;
            crate::ast::node_data_generated::for_each_child(n, |child| {
                if walk(child) {
                    found = true;
                    true
                } else {
                    false
                }
            });
            found
        }
        walk(body)
    }


    pub(crate) fn has_same_named_type_symbol(&self, name: &str) -> bool {
        let type_meaning = SymbolFlags::Interface
            | SymbolFlags::Class
            | SymbolFlags::TypeParameter
            | SymbolFlags::TypeAlias
            | SymbolFlags::RegularEnum
            | SymbolFlags::ConstEnum;
        let symbol_map = self.program.symbol_map();
        for &container_id in self.scope_stack.iter().rev() {
            if let Some(locals) = symbol_map.locals.get(&container_id)
                && let Some(sym) = locals.get(name)
                && sym.flags.intersects(type_meaning)
            {
                return true;
            }
            if let Some(container_sym) = symbol_map.symbols.get(&container_id)
                && (container_sym
                    .members
                    .get(name)
                    .is_some_and(|s| s.flags.intersects(type_meaning))
                    || container_sym
                        .exports
                        .get(name)
                        .is_some_and(|s| s.flags.intersects(type_meaning)))
            {
                return true;
            }
        }
        self.globals
            .get(name)
            .is_some_and(|s| s.flags.intersects(type_meaning))
    }

    pub(crate) fn namespace_usable_as_value(&mut self, namespace: &Arc<Symbol>) -> bool {
        let state_instantiated = namespace
            .declarations
            .iter()
            .filter(|d| d.kind == SyntaxKind::ModuleDeclaration)
            .any(|d| {
                module_is_instantiated(d, self.compiler_options.should_preserve_const_enums())
            });
        state_instantiated || self.namespace_has_value_side(namespace)
    }

    pub(crate) fn namespace_has_value_side(&mut self, namespace: &Arc<Symbol>) -> bool {
        let value_flags = SymbolFlags::Function
            | SymbolFlags::Class
            | SymbolFlags::FunctionScopedVariable
            | SymbolFlags::BlockScopedVariable
            | SymbolFlags::RegularEnum
            | SymbolFlags::ConstEnum
            | SymbolFlags::Method;
        let has_value_member = |table: &crate::ast::SymbolTable| {
            table.iter().any(|(name, s)| {
                name != "export="
                    && s.flags.intersects(value_flags)

                    && s.declarations.iter().any(|d| {
                        !matches!(
                            d.kind,
                            SyntaxKind::Parameter | SyntaxKind::MethodSignature
                        )
                    })
            })
        };
        if has_value_member(&namespace.exports) || has_value_member(&namespace.members) {
            return true;
        }

        for d in &namespace.declarations {
            if d.kind != SyntaxKind::ModuleDeclaration {
                continue;
            }
            let entries: Vec<(String, Arc<Symbol>)> = self
                .program
                .symbol_map()
                .locals
                .get(&d.id())
                .map(|table| {
                    table
                        .iter()
                        .map(|(k, v)| (k.clone(), Arc::clone(v)))
                        .collect()
                })
                .unwrap_or_default();
            if entries.iter().any(|(name, s)| {
                name != "export="
                    && (s.flags.intersects(value_flags)
                        || (s.flags.contains(SymbolFlags::ValueModule)
                            && self.namespace_has_value_side(s)))
            }) {
                return true;
            }
        }

        if self.namespace_value_depth < 4 {
            self.namespace_value_depth += 1;
            let nested = namespace
                .exports
                .iter()
                .chain(namespace.members.iter())
                .any(|(name, s)| {
                    name != "export="
                        && s.flags.contains(SymbolFlags::ValueModule)
                        && self.namespace_has_value_side(s)
                });
            self.namespace_value_depth -= 1;
            if nested {
                return true;
            }
        }

        for decl in &namespace.declarations {
            if decl.kind == SyntaxKind::ModuleDeclaration
                && let Some(locals) = self.program.symbol_map().locals.get(&decl.id())
                && locals.iter().any(|(name, s)| {
                    name != "export=" && s.flags.intersects(value_flags)
                })
            {
                return true;
            }
        }

        if let Some(export_equals) = namespace.exports.get("export=") {
            for decl in &export_equals.declarations {
                if let crate::ast::NodeData::ExportAssignment(ea) = &decl.data
                    && ea.is_export_equals
                    && matches!(
                        ea.expression.kind,
                        SyntaxKind::Identifier | SyntaxKind::QualifiedName
                    )
                {

                    let scope_decl = namespace
                        .declarations
                        .iter()
                        .find(|d| d.kind == SyntaxKind::ModuleDeclaration)
                        .cloned();
                    if let Some(scope_decl) = scope_decl {
                        self.push_scope(&scope_decl);
                        let target = self.resolve_qualified_symbol(&ea.expression);
                        self.pop_scope();
                        if let Some(target) = target {
                            if target.flags.intersects(value_flags) {
                                return true;
                            }
                            if target.flags.contains(SymbolFlags::ValueModule) {
                                return self.namespace_has_value_side(&target);
                            }
                        }
                    }
                }
            }
        }
        false
    }

    pub(crate) fn resolve_module_member_symbol(
        &mut self,
        module_sym: &Arc<Symbol>,
        name: &str,
        depth: usize,
    ) -> Option<Arc<Symbol>> {
        if depth == 0 {
            return None;
        }
        let sym = self.namespace_member_recursive(module_sym, name);
        if let Some(sym) = &sym {

            if let Some(target) = &sym.export_symbol
                && !Arc::ptr_eq(target, &sym)
            {
                return Some(Arc::clone(target));
            }
        }

        let mut clause_hits: Vec<(String, Option<String>)> = Vec::new();
        self.for_each_module_statement(module_sym, |stmt| {
            if let crate::ast::NodeData::ExportDeclaration(d) = &stmt.data
                && let Some(clause) = &d.export_clause
                && let crate::ast::NodeData::NamedExports(ne) = &clause.data
            {
                for el in ne.elements.iter() {
                    if let crate::ast::NodeData::ExportSpecifier(spec) = &el.data
                        && spec.name.text().trim_matches(['"', '\'', '`']) == name
                    {
                        let imported = spec
                            .property_name
                            .as_ref()
                            .unwrap_or(&spec.name)
                            .text()
                            .trim_matches(['"', '\'', '`'])
                            .to_string();
                        let module_text = d.module_specifier.as_ref().map(|module_spec| {
                            module_spec
                                .text()
                                .trim_matches(['"', '\'', '`'])
                                .to_string()
                        });
                        clause_hits.push((imported, module_text));
                        return true;
                    }
                }
            }
            false
        });
        for (imported, module_text) in clause_hits {
            let target_module = match module_text {

                None => Arc::clone(module_sym),
                Some(text) => match self.resolve_module_spec_from(module_sym, &text) {
                    Some(m) => m,
                    None => continue,
                },
            };
            if let Some(target) =
                self.resolve_module_member_symbol(&target_module, &imported, depth - 1)
            {
                return Some(target);
            }
        }
        sym
    }

    pub(crate) fn resolve_module_spec_from(
        &self,
        base_module: &Arc<Symbol>,
        specifier: &str,
    ) -> Option<Arc<Symbol>> {
        if !specifier.starts_with('.') {
            return self.resolve_module_file_symbol(specifier);
        }
        let dir = base_module
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::SourceFile)
            .and_then(|d| self.get_source_file_of_node(d))
            .map(|f| {
                f.file_name
                    .rfind('/')
                    .map(|i| f.file_name[..i].to_string())
                    .unwrap_or_default()
            })?;
        self.resolve_module_file_symbol_in(&dir, specifier)
    }

    pub(crate) fn type_of_dynamic_import(&mut self, node: &Arc<Node>) -> Option<Arc<Type>> {
        let spec = self.spec_of_dynamic_import_call(node)?;
        if spec.is_empty() {
            return None;
        }
        let cur = self.current_file.clone()?;

        let module_sym = match self.resolve_module_file_symbol(&spec) {
            Some(s) => s,
            None => {
                let path = self.program.resolve_external_module_path(
                    &spec,
                    &cur.file_name,
                    crate::core::compiler_options::ModuleKind::ESNext,
                )?;
                let sf = self.program.get_source_file(&path)?;
                self.program.symbol_map().symbol_of(&sf.node).cloned()?
            }
        };
        Some(self.resolve_namespace_type(&module_sym))
    }

    pub(crate) fn spec_of_dynamic_import_call(&self, node: &Arc<Node>) -> Option<String> {
        if node.kind != SyntaxKind::CallExpression {
            return None;
        }
        let (callee, args) = match &node.data {
            NodeData::CallExpression(d) => (&d.expression, &d.arguments),
            _ => return None,
        };
        if callee.kind != SyntaxKind::ImportKeyword {
            return None;
        }
        let spec_node = args.iter().next()?;
        if spec_node.kind != SyntaxKind::StringLiteral {
            return None;
        }
        Some(spec_node.text().trim_matches(['"', '\'', '`']).to_string())
    }

    pub(crate) fn type_of_imported_symbol(&mut self, symbol: &Arc<Symbol>) -> Option<Arc<Type>> {

        if let Some(decl) = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ImportEqualsDeclaration)
        {
            let crate::ast::NodeData::ImportEqualsDeclaration(ied) = &decl.data else {
                return None;
            };
            if ied.module_reference.kind == SyntaxKind::ExternalModuleReference {
                let ext = &ied.module_reference;
                let crate::ast::NodeData::ExternalModuleReference(emr) = &ext.data else {
                    return None;
                };
                let module_spec = emr.expression.text().to_string();
                let module_text_trimmed = module_spec.trim_matches(['"', '\'', '`']).to_string();
                let module_sym = match self.resolve_module_file_symbol(&module_spec) {
                    Some(s) => s,
                    None => {
                        let Some(cur) = self.current_file.clone() else {
                            return None;
                        };
                        let Some(path) = self.program.resolve_external_module_path(
                            &module_text_trimmed,
                            &cur.file_name,
                            crate::core::compiler_options::ModuleKind::None,
                        ) else {
                            return None;
                        };
                        let Some(sf) = self.program.get_source_file(&path) else {
                            return None;
                        };
                        let Some(sym) =
                            self.program.symbol_map().symbol_of(&sf.node).cloned()
                        else {
                            return None;
                        };
                        sym
                    }
                };

                if let Some(eq) =
                    module_sym.exports.get(crate::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)
                {
                    let entity_decl = eq
                        .declarations
                        .iter()
                        .find(|d| d.kind == SyntaxKind::ExportAssignment)
                        .cloned();
                    let scope_decl = module_sym
                        .declarations
                        .iter()
                        .find(|d| d.kind == SyntaxKind::ModuleDeclaration)
                        .cloned();
                    if let Some(export_decl) = entity_decl
                        && let crate::ast::NodeData::ExportAssignment(ea) = &export_decl.data
                        && ea.is_export_equals
                        && matches!(
                            ea.expression.kind,
                            SyntaxKind::Identifier | SyntaxKind::QualifiedName
                        )
                    {
                        if let Some(scope) = scope_decl {
                            self.push_scope(&scope);
                            let target = self.resolve_qualified_symbol(&ea.expression);
                            self.pop_scope();
                            if let Some(t) = target {
                                return Some(self.get_type_of_symbol(&t));
                            }
                        } else {

                            let mut segments: Vec<String> = Vec::new();
                            let mut cur = &ea.expression;
                            loop {
                                match &cur.data {
                                    crate::ast::NodeData::Identifier(id) => {
                                        segments.push(id.text.clone());
                                        break;
                                    }
                                    crate::ast::NodeData::QualifiedName(q) => {
                                        segments.push(q.right.text().to_string());
                                        cur = &q.left;
                                    }
                                    _ => break,
                                }
                            }
                            segments.reverse();
                            if let Some(first) = segments.first()
                                && let Some(mut target) =
                                    self.resolve_module_member_symbol(&module_sym, first, 8)
                            {
                                let mut ok = true;
                                for seg in segments.iter().skip(1) {
                                    match target
                                        .exports
                                        .get(seg)
                                        .or_else(|| target.members.get(seg))
                                        .cloned()
                                    {
                                        Some(next) => target = next,
                                        None => {
                                            ok = false;
                                            break;
                                        }
                                    }
                                }
                                if ok {
                                    return Some(self.get_type_of_symbol(&target));
                                }
                            }
                        }
                    }
                }
                return Some(self.resolve_namespace_type(&module_sym));
            }

            let target = &ied.module_reference;
            let t = self.get_type_of_node(target);
            if t.flags.contains(TypeFlags::Any)
                && t.intrinsic_name() == Some("any")
                && self.resolve_identifier(target).is_none()
            {
                return None;
            }
            return Some(t);
        }
        let decl = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ImportSpecifier)?;
        let name = match &decl.data {

            crate::ast::NodeData::ImportSpecifier(d) => d
                .property_name
                .as_ref()
                .map_or_else(|| d.name.text().to_string(), |p| p.text().to_string()),
            _ => return None,
        };

        let mut import_decl = decl.parent.as_ref()?;
        while !matches!(import_decl.data, crate::ast::NodeData::ImportDeclaration(_)) {
            import_decl = import_decl.parent.as_ref()?;
        }
        let module_spec = match &import_decl.data {
            crate::ast::NodeData::ImportDeclaration(d) => d.module_specifier.text().to_string(),
            _ => return None,
        };
        let module_text_trimmed = module_spec.trim_matches(['"', '\'', '`']).to_string();
        let module_sym = match self.resolve_module_file_symbol(&module_spec) {
            Some(s) => s,
            None => {

                let Some(cur) = self.current_file.clone() else {
                    return None;
                };
                let Some(path) = self.program.resolve_external_module_path(
                    &module_text_trimmed,
                    &cur.file_name,
                    crate::core::compiler_options::ModuleKind::None,
                ) else {
                    return None;
                };
                let Some(sf) = self.program.get_source_file(&path) else {
                    return None;
                };
                let Some(sym) = self.program.symbol_map().symbol_of(&sf.node).cloned() else {
                    return None;
                };
                sym
            }
        };
        let Some(member) = self.resolve_module_member_symbol(&module_sym, &name, 8) else {

            if name == "default"
                && self.program.options().allow_synthetic_default_imports.is_true()
            {
                return Some(self.get_any_type());
            }
            return None;
        };
        if let Some(t) = self
            .value_symbol_links
            .get(&member)
            .and_then(|l| l.resolved_type.clone())
        {
            return Some(t);
        }
        for d in &member.declarations {
            match d.kind {
                SyntaxKind::FunctionDeclaration => {
                    return Some(self.get_type_of_function_like(d));
                }
                SyntaxKind::ClassDeclaration => {
                    return Some(self.get_type_of_class_declaration(d));
                }
                _ => {}
            }
        }

        Some(self.get_type_of_symbol(&member))
    }

    pub(crate) fn object_literal_export_member(
        &self,
        namespace: &Arc<Symbol>,
        name: &str,
    ) -> Option<Arc<Symbol>> {
        let ea_sym = namespace.exports.get("export=")?;
        for d in &ea_sym.declarations {
            if let crate::ast::NodeData::ExportAssignment(ea) = &d.data
                && ea.is_export_equals
                && let crate::ast::NodeData::ObjectLiteralExpression(ol) = &ea.expression.data
            {
                for prop in ol.properties.iter() {
                    if prop.text() == name
                        && let Some(s) = self.program.symbol_map().symbol_of(prop)
                    {
                        return Some(Arc::clone(s));
                    }
                }
            }
        }
        None
    }

    pub(crate) fn heritage_type_arguments_for_base(
        &mut self,
        base_sym: &Arc<Symbol>,
    ) -> Option<Vec<Arc<Type>>> {
        let class_node = self.enclosing_class_stack.last().cloned()?;
        let heritage = match &class_node.data {
            crate::ast::NodeData::ClassDeclaration(data) => data.heritage_clauses.clone(),
            _ => return None,
        };
        for clause in heritage?.iter() {
            let crate::ast::NodeData::HeritageClause(hc) = &clause.data else {
                continue;
            };
            if hc.token != SyntaxKind::ExtendsKeyword {
                continue;
            }
            for type_ref in hc.types.iter() {
                let crate::ast::NodeData::ExpressionWithTypeArguments(ewa) = &type_ref.data
                else {
                    continue;
                };
                let type_args = ewa.type_arguments.as_ref()?;
                if ewa.expression.kind == SyntaxKind::Identifier
                    && let Some(sym) = self.resolve_identifier(&ewa.expression)
                    && Arc::ptr_eq(&sym, base_sym)
                {
                    return Some(
                        type_args
                            .iter()
                            .map(|t| self.get_type_from_type_node(t))
                            .collect(),
                    );
                }
            }
        }
        None
    }

    pub(crate) fn namespace_member_recursive(
        &mut self,
        namespace: &Arc<Symbol>,
        name: &str,
    ) -> Option<Arc<Symbol>> {
        if let Some(s) = namespace.exports.get(name).or_else(|| namespace.members.get(name)) {
            return Some(Arc::clone(s));
        }
        for d in &namespace.declarations {
            if d.kind == SyntaxKind::ModuleDeclaration
                && let Some(s) = self
                    .program
                    .symbol_map()
                    .locals
                    .get(&d.id())
                    .and_then(|l| l.get(name))
            {
                return Some(Arc::clone(s));
            }
        }

        let export_equals = namespace.exports.get("export=")?;
        for d in &export_equals.declarations {
            if let crate::ast::NodeData::ExportAssignment(ea) = &d.data
                && ea.is_export_equals
            {

                if let crate::ast::NodeData::ObjectLiteralExpression(ol) = &ea.expression.data {
                    for prop in ol.properties.iter() {
                        if prop.text() == name
                            && let Some(s) = self.program.symbol_map().symbol_of(prop)
                        {
                            return Some(Arc::clone(s));
                        }
                    }
                    continue;
                }
                if matches!(
                    ea.expression.kind,
                    SyntaxKind::Identifier | SyntaxKind::QualifiedName
                )
                {
                let scope_decl = namespace
                    .declarations
                    .iter()
                    .find(|d| d.kind == SyntaxKind::ModuleDeclaration)
                    .cloned();
                let target = scope_decl.and_then(|scope_decl| {
                    self.push_scope(&scope_decl);
                    let t = self.resolve_qualified_symbol(&ea.expression);
                    self.pop_scope();
                    t
                });
                if let Some(mut target) = target {

                    for _ in 0..4 {
                        if target.flags.contains(SymbolFlags::ValueModule) {
                            break;
                        }
                        if target.flags != SymbolFlags::Alias {
                            break;
                        }
                        let next = target
                            .declarations
                            .iter()
                            .find(|d| d.kind == SyntaxKind::ImportEqualsDeclaration)
                            .and_then(|d| {
                                if let crate::ast::NodeData::ImportEqualsDeclaration(ied) = &d.data
                                    && matches!(
                                        ied.module_reference.kind,
                                        SyntaxKind::Identifier | SyntaxKind::QualifiedName
                                    )
                                {
                                    Some(self.resolve_qualified_symbol(&ied.module_reference))
                                } else {
                                    None
                                }
                            })
                            .flatten();
                        match next {
                            Some(n) => target = n,
                            None => break,
                        }
                    }
                    if target.flags.contains(SymbolFlags::ValueModule) {
                        return self.namespace_member_recursive(&target, name);
                    }
                    return Some(target);
                }
                }
            }
        }
        None
    }

    pub(crate) fn namespace_full_path(symbol: &Arc<Symbol>) -> String {
        let decl = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ModuleDeclaration);
        let Some(decl) = decl else {
            return symbol.name.clone();
        };
        let mut parts: Vec<String> = Vec::new();
        let mut current: Option<&Arc<Node>> = Some(decl);
        while let Some(n) = current {
            if let crate::ast::NodeData::ModuleDeclaration(md) = &n.data {
                parts.push(md.name.text().trim_matches(['"', '\'']).to_string());
            }
            current = n.parent.as_ref();
        }
        parts.reverse();
        parts.join(".")
    }
}
