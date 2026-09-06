use std::sync::Arc;

use crate::ast::{
    ModifierFlags, Node,
    NodeFlags, NodeList, Symbol, SymbolFlags, SyntaxKind,
};







use super::*;


impl Checker {
    pub(crate) fn find_name_suggestion(&self, name: &str, meaning: SymbolFlags) -> Option<String> {

        let mut candidates: Vec<&Arc<Symbol>> = Vec::new();
        let symbol_map = self.program.symbol_map();
        fn push_symbol<'a>(
            cands: &mut Vec<&'a Arc<Symbol>>,
            sym: &'a Arc<Symbol>,
            meaning: SymbolFlags,
        ) {
            if sym.flags.intersects(meaning) {
                cands.push(sym);
            }
        }

        if let Some(file) = self.current_file.as_ref() {
            let fid = file.id();
            if let Some(locals) = symbol_map.locals.get(&fid) {
                for sym in locals.entries.values() {
                    push_symbol(&mut candidates, sym, meaning);
                }
            }

            if let Some(sym) = symbol_map.symbols.get(&fid) {
                for sub in sym.members.entries.values() {
                    push_symbol(&mut candidates, sub, meaning);
                }
                for sub in sym.exports.entries.values() {
                    push_symbol(&mut candidates, sub, meaning);
                }
            }
        }
        for &container_id in self.scope_stack.iter() {
            if let Some(locals) = symbol_map.locals.get(&container_id) {
                for sym in locals.entries.values() {
                    push_symbol(&mut candidates, sym, meaning);
                }
            }
            if let Some(sym) = symbol_map.symbols.get(&container_id) {
                for sub in sym.members.entries.values() {
                    push_symbol(&mut candidates, sub, meaning);
                }
                for sub in sym.exports.entries.values() {
                    push_symbol(&mut candidates, sub, meaning);
                }
            }
        }
        for sym in self.globals.entries.values() {
            push_symbol(&mut candidates, sym, meaning);
        }

        let rune_len = name.chars().count();
        let maximum_length_difference = ((rune_len as f64) * 0.34) as usize;
        let maximum_length_difference = maximum_length_difference.max(2);
        let mut best_distance = ((rune_len as f64) * 0.4).floor() + 0.9;
        let mut best: Option<((usize, usize), &String)> = None;
        for sym in candidates {
            let cand: &String = &sym.name;

            if cand.is_empty()
                || cand.starts_with('"')
                || cand.starts_with('\'')
                || cand.starts_with('`')
                || cand.starts_with('\u{FE}')
            {
                continue;
            }
            let cand_len = cand.chars().count();
            if cand_len < 3 && !cand.eq_ignore_ascii_case(name) {
                continue;
            }
            if rune_len.max(cand_len) - rune_len.min(cand_len) > maximum_length_difference {
                continue;
            }
            if cand == name {
                continue;
            }
            let Some(d) = levenshtein_with_max(name, cand, best_distance) else {
                continue;
            };

            let key = self.suggestion_order_key(sym);
            let replace = match &best {
                None => true,
                Some((bkey, _)) => {
                    if d < best_distance {
                        true
                    } else {
                        key < *bkey
                    }
                }
            };
            if d < best_distance {
                best_distance = d;
            }
            if replace {
                best = Some((key, cand));
            }
        }
        best.map(|(_, c)| c.clone())
    }

    pub(crate) fn suggestion_order_key(&self, sym: &Arc<Symbol>) -> (usize, usize) {
        let Some(decl) = sym.declarations.first() else {
            return (usize::MAX, usize::MAX);
        };
        let Some(sf) = self.get_source_file_of_node(decl) else {
            return (usize::MAX, usize::MAX);
        };
        let idx = self
            .files
            .iter()
            .position(|f| f.node.id() == sf.node.id())
            .unwrap_or(usize::MAX);
        (idx, decl.loc.pos())
    }

    pub(crate) fn inside_function_body(node: &Arc<Node>) -> bool {
        let mut anc = node.parent.as_ref();
        while let Some(a) = anc {
            match a.kind {
                SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionExpression
                | SyntaxKind::ArrowFunction
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::Constructor
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor => return true,
                SyntaxKind::ModuleBlock | SyntaxKind::SourceFile | SyntaxKind::ModuleDeclaration => {
                    return false
                }
                _ => {}
            }
            anc = a.parent.as_ref();
        }
        false
    }

    pub(crate) fn check_class_heritage_members(&mut self, node: &Arc<Node>) {
        let crate::ast::NodeData::ClassDeclaration(data) = &node.data else {
            return;
        };
        let Some((base_node, _base_sym)) = self.extends_base_of(node) else {
            return;
        };
        let class_name = data
            .name
            .as_ref()
            .map(|n| n.text().to_string())
            .unwrap_or_default();
        let base_name = Self::class_name_text(&base_node);

        if !node.has_syntactic_modifier(ModifierFlags::Abstract) {
            let mut missing: Vec<String> = Vec::new();
            Self::collect_unimplemented_abstract_members(node, &base_node, &mut missing);
            missing.dedup();
            if !missing.is_empty() {
                let file = self.current_file.clone();
                let name_loc = data
                    .name
                    .as_ref()
                    .map(|n| n.loc)
                    .unwrap_or(node.loc);
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    name_loc,
                    crate::diagnostics::messages_generated::
                        NON_ABSTRACT_CLASS_0_IS_MISSING_IMPLEMENTATIONS_FOR_THE_FOLLOWING_MEMBERS_OF_1_COLON_2,
                    vec![
                        class_name.clone(),
                        base_name.clone(),
                        missing
                            .iter()
                            .map(|m| format!("'{m}'"))
                            .collect::<Vec<_>>()
                            .join(", "),
                    ],
                ));
            }
        }

        for member in data.members.iter() {
            let (name_node, own_type): (&Arc<Node>, Option<Arc<Type>>) = match &member.data {
                crate::ast::NodeData::PropertyDeclaration(pd) => {
                    if pd.name.kind != SyntaxKind::Identifier {
                        continue;
                    }
                    let t = if let Some(tn) = &pd.type_node {
                        Some(self.get_type_from_type_node(tn))
                    } else {
                        pd.initializer
                            .as_ref()
                            .map(|init| self.get_type_of_node(init))
                    };
                    (&pd.name, t)
                }
                crate::ast::NodeData::GetAccessorDeclaration(gd) => {
                    if gd.name.kind != SyntaxKind::Identifier {
                        continue;
                    }

                    let t = if let Some(tn) = &gd.type_node {
                        Some(self.get_type_from_type_node(tn))
                    } else {
                        Self::first_return_expression(gd.body.as_ref())
                            .map(|e| self.get_type_of_node(&e))
                    };
                    (&gd.name, t)
                }
                _ => continue,
            };
            let Some(own_type) = own_type else { continue };
            let prop_name = name_node.text().to_string();
            let Some(base_member) = Self::find_class_member_by_name(&base_node, &prop_name)
            else {
                continue;
            };
            let base_tn = match &base_member.data {
                crate::ast::NodeData::PropertyDeclaration(pd) => pd.type_node.clone(),
                crate::ast::NodeData::GetAccessorDeclaration(gd) => gd.type_node.clone(),
                crate::ast::NodeData::SetAccessorDeclaration(sd) => sd
                    .parameters
                    .iter()
                    .next()
                    .and_then(|p| {
                        if let crate::ast::NodeData::ParameterDeclaration(pd) = &p.data {
                            pd.type_node.clone()
                        } else {
                            None
                        }
                    }),
                _ => None,
            };
            let Some(base_tn) = base_tn else {
                continue;
            };
            let base_type = self.get_type_from_type_node(&base_tn);
            if !own_type.flags.contains(TypeFlags::Any)
                && !self.is_type_assignable_to(&own_type, &base_type)
            {
                let file = self.current_file.clone();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    name_node.loc,
                    crate::diagnostics::messages_generated::
                        PROPERTY_0_IN_TYPE_1_IS_NOT_ASSIGNABLE_TO_THE_SAME_PROPERTY_IN_BASE_TYPE_2,
                    vec![
                        prop_name,
                        class_name.clone(),
                        base_name.clone(),
                    ],
                ));
            }
        }
    }

    pub(crate) fn class_members_of(class: &Arc<Node>) -> &Arc<NodeList> {
        match &class.data {
            crate::ast::NodeData::ClassDeclaration(d) => &d.members,
            crate::ast::NodeData::ClassExpression(d) => &d.members,
            _ => {
                static EMPTY: std::sync::OnceLock<Arc<NodeList>> = std::sync::OnceLock::new();
                EMPTY.get_or_init(|| Arc::new(NodeList::default()))
            }
        }
    }

    pub(crate) fn find_class_member_by_name(class: &Arc<Node>, name: &str) -> Option<Arc<Node>> {
        Self::class_members_of(class)
            .iter()
            .find(|m| {
                let n = match &m.data {
                    crate::ast::NodeData::PropertyDeclaration(d) => &d.name,
                    crate::ast::NodeData::MethodDeclaration(d) => &d.name,
                    crate::ast::NodeData::GetAccessorDeclaration(d) => &d.name,
                    crate::ast::NodeData::SetAccessorDeclaration(d) => &d.name,
                    _ => return false,
                };
                n.kind == SyntaxKind::Identifier && n.text() == name
            })
            .cloned()
    }

    pub(crate) fn collect_unimplemented_abstract_members(
        class: &Arc<Node>,
        base: &Arc<Node>,
        out: &mut Vec<String>,
    ) {
        for member in Self::class_members_of(base).iter() {
            let (name_node, is_abstract_member) = match &member.data {
                crate::ast::NodeData::PropertyDeclaration(d) => {
                    (&d.name, member.has_syntactic_modifier(ModifierFlags::Abstract))
                }
                crate::ast::NodeData::MethodDeclaration(d) => {
                    (&d.name, member.has_syntactic_modifier(ModifierFlags::Abstract))
                }
                crate::ast::NodeData::GetAccessorDeclaration(d) => (
                    &d.name,
                    member.has_syntactic_modifier(ModifierFlags::Abstract),
                ),
                crate::ast::NodeData::SetAccessorDeclaration(d) => (
                    &d.name,
                    member.has_syntactic_modifier(ModifierFlags::Abstract),
                ),
                _ => continue,
            };
            if name_node.kind != SyntaxKind::Identifier {
                continue;
            }
            let name = name_node.text();
            if is_abstract_member {

                if !Self::chain_implements(class, name) {
                    out.push(name.to_string());
                }
            } else if out.iter().any(|m| m == name) {

                out.retain(|m| m != name);
            }
        }
    }

    pub(crate) fn first_return_expression(body: Option<&Arc<Node>>) -> Option<Arc<Node>> {
        fn walk(n: &Arc<Node>) -> Option<Arc<Node>> {
            if let crate::ast::NodeData::ReturnStatement(d) = &n.data
                && let Some(e) = &d.expression
            {
                return Some(Arc::clone(e));
            }
            let mut found: Option<Arc<Node>> = None;
            crate::ast::node_data_generated::for_each_child(n, |child| {
                if found.is_none() {
                    found = walk(child);
                }
                found.is_some()
            });
            found
        }
        body.and_then(walk)
    }

    pub(crate) fn chain_implements(class: &Arc<Node>, name: &str) -> bool {
        for member in Self::class_members_of(class).iter() {
            let (name_node, is_abstract) = match &member.data {
                crate::ast::NodeData::PropertyDeclaration(d) => {
                    (&d.name, member.has_syntactic_modifier(ModifierFlags::Abstract))
                }
                crate::ast::NodeData::MethodDeclaration(d) => {
                    (&d.name, member.has_syntactic_modifier(ModifierFlags::Abstract))
                }
                crate::ast::NodeData::GetAccessorDeclaration(d) => (
                    &d.name,
                    member.has_syntactic_modifier(ModifierFlags::Abstract),
                ),
                crate::ast::NodeData::SetAccessorDeclaration(d) => (
                    &d.name,
                    member.has_syntactic_modifier(ModifierFlags::Abstract),
                ),
                _ => continue,
            };
            if name_node.kind == SyntaxKind::Identifier
                && name_node.text() == name
                && !is_abstract
            {
                return true;
            }
        }

        false
    }

    pub(crate) fn assignments_to_name(
        body: &Arc<Node>,
        name: &str,
    ) -> Vec<(crate::core::text::TextRange, Arc<Node>)> {
        let mut found = Vec::new();
        fn walk(n: &Arc<Node>, name: &str, found: &mut Vec<(crate::core::text::TextRange, Arc<Node>)>) {
            if let crate::ast::NodeData::BinaryExpression(data) = &n.data
                && data.operator_token.kind == SyntaxKind::EqualsToken
                && data.left.kind == SyntaxKind::Identifier
                && data.left.text() == name
            {
                found.push((data.left.loc, Arc::clone(&data.right)));
            }
            crate::ast::node_data_generated::for_each_child(n, |child| {
                walk(child, name, found);
                false
            });
        }
        walk(body, name, &mut found);
        found
    }

    pub fn resolve_qualified_symbol(&mut self, name: &Arc<Node>) -> Option<Arc<Symbol>> {
        match self.resolve_qualified_symbol_traced(name) {
            Ok(s) => Some(s),
            Err(_) => None,
        }
    }

    pub fn resolve_qualified_symbol_traced(
        &mut self,
        name: &Arc<Node>,
    ) -> Result<Arc<Symbol>, (Arc<Node>, String, String)> {
        match &name.data {
            crate::ast::NodeData::Identifier(_) => match self.resolve_identifier(name) {
                Some(s) => Ok(s),
                None => Err((Arc::clone(name), String::new(), String::new())),
            },
            crate::ast::NodeData::QualifiedName(data) => {
                self.resolve_qualified_tail(&data.left, &data.right)
            }

            crate::ast::NodeData::PropertyAccessExpression(pa) => {
                let mut base = &pa.expression;
                while let crate::ast::NodeData::ParenthesizedExpression(p) = &base.data {
                    base = &p.expression;
                }
                if matches!(
                    base.kind,
                    SyntaxKind::Identifier
                        | SyntaxKind::QualifiedName
                        | SyntaxKind::PropertyAccessExpression
                ) {
                    self.resolve_qualified_tail(base, &pa.name)
                } else {
                    Err((Arc::clone(name), String::new(), String::new()))
                }
            }
            _ => Err((Arc::clone(name), String::new(), String::new())),
        }
    }

    pub(crate) fn resolve_qualified_tail(
        &mut self,
        left: &Arc<Node>,
        right: &Arc<Node>,
    ) -> Result<Arc<Symbol>, (Arc<Node>, String, String)> {
        {
            let mut symbol = self.resolve_qualified_symbol_traced(left)?;
            let path_so_far = qualified_name_text(left);
            symbol = self.resolve_alias_base(symbol);

            if symbol.flags == SymbolFlags::Alias
                && let Some(module_sym) = self.resolve_import_alias_module(&symbol)
            {
                symbol = module_sym;
            }

            let text = right.text();
            let mut next = symbol
                .exports
                .get(text)
                .or_else(|| symbol.members.get(text))
                .cloned()
                .or_else(|| self.ambient_namespace_local(&symbol, text))

                .or_else(|| self.object_literal_export_member(&symbol, text));

                if next.is_none()
                    && let Some(ea_sym) = symbol.exports.get("export=")
                    && let Some(decl) = ea_sym
                        .declarations
                        .iter()
                        .find(|d| d.kind == SyntaxKind::ExportAssignment)
                    && let crate::ast::NodeData::ExportAssignment(ea) = &decl.data
                    && ea.is_export_equals
                    && matches!(
                        ea.expression.kind,
                        SyntaxKind::Identifier | SyntaxKind::QualifiedName
                    )
                {

                    let scope = symbol
                        .declarations
                        .iter()
                        .find(|d| d.kind == SyntaxKind::ModuleDeclaration)
                        .cloned();
                    if let Some(scope) = scope {
                        self.push_scope(&scope);
                        let target = self.resolve_identifier(&ea.expression);
                        self.pop_scope();
                        if let Some(target) = target
                            && target.flags.contains(SymbolFlags::ValueModule)
                        {
                            next = target
                                .exports
                                .get(text)
                                .or_else(|| target.members.get(text))
                                .cloned()
                                .or_else(|| self.ambient_namespace_local(&target, text));
                        }
                    }
                }

                let base_is_unresolved_require_alias = symbol.flags == SymbolFlags::Alias
                    && symbol
                        .declarations
                        .iter()
                        .any(|d| {
                            if let crate::ast::NodeData::ImportEqualsDeclaration(ied) = &d.data
                                && let crate::ast::NodeData::ExternalModuleReference(ext) =
                                    &ied.module_reference.data
                                && ext.expression.kind == SyntaxKind::StringLiteral
                            {
                                self.resolve_module_file_symbol(&ext.expression.text()).is_none()
                            } else {
                                false
                            }
                        });
                if base_is_unresolved_require_alias {
                    return Ok(symbol);
                }
                match next {
                    Some(next) => {

                        let resolved = if next.flags.intersects(SymbolFlags::Alias) {
                            let scope = symbol
                                .declarations
                                .iter()
                                .find(|d| {
                                    d.kind == SyntaxKind::ModuleDeclaration
                                        || d.kind == SyntaxKind::SourceFile
                                })
                                .cloned();
                            if let Some(ref scope) = scope {
                                self.push_scope(scope);
                            }
                            let base = self.resolve_alias_base(Arc::clone(&next));
                            if scope.is_some() {
                                self.pop_scope();
                            }
                            base
                        } else {
                            match self.follow_alias(&next) {
                                Some(f) => f,
                                None => next,
                            }
                        };
                        Ok(resolved)
                    }
                    None => {
                        let _ = path_so_far;
                        Err((
                            Arc::clone(right),
                            Self::namespace_full_path(&symbol),
                            text.to_string(),
                        ))
                    }
                }
            }
    }

    pub(crate) fn ambient_ancestor(&self, node: &Arc<Node>) -> bool {
        let mut cur = node.parent.as_ref();
        while let Some(a) = cur {
            if a.has_syntactic_modifier(ModifierFlags::Ambient) {
                return true;
            }
            cur = a.parent.as_ref();
        }
        false
    }

    pub(crate) fn ambient_namespace_locals_visible(&self, ns: &Arc<Symbol>) -> bool {
        if std::env::var_os("TSOX_NO_AMBIENT").is_some() {
            return false;
        }
        ns.declarations.iter().any(|d| {
            d.kind == SyntaxKind::ModuleDeclaration
                && (d.has_syntactic_modifier(ModifierFlags::Ambient)
                    || self.ambient_ancestor(d)
                    || self
                        .get_source_file_of_node(d)
                        .is_some_and(|f| f.is_declaration_file))
                && !crate::binder::Binder::has_export_declarations(d)
        })
    }

    pub(crate) fn ambient_namespace_local(&self, ns: &Arc<Symbol>, name: &str) -> Option<Arc<Symbol>> {
        if !self.ambient_namespace_locals_visible(ns) {
            return None;
        }
        ns.declarations
            .iter()
            .filter(|d| d.kind == SyntaxKind::ModuleDeclaration)
            .find_map(|d| {
                self.program
                    .symbol_map()
                    .locals
                    .get(&d.id())
                    .and_then(|l| l.get(name))
                    .cloned()
            })
    }

    pub(crate) fn resolve_alias_base(&mut self, symbol: Arc<Symbol>) -> Arc<Symbol> {
        if !symbol.flags.intersects(SymbolFlags::Alias) {
            return symbol;
        }

        if symbol
            .declarations
            .iter()
            .any(|d| matches!(d.kind, SyntaxKind::NamespaceImport | SyntaxKind::NamespaceExport))
            && let Some(module_sym) = self.resolve_import_alias_module(&symbol)
        {
            return module_sym;
        }
        if let Some(decl) = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ImportEqualsDeclaration)
        {
            if let crate::ast::NodeData::ImportEqualsDeclaration(data) = &decl.data {

                if let crate::ast::NodeData::ExternalModuleReference(ext) =
                    &data.module_reference.data
                    && ext.expression.kind == SyntaxKind::StringLiteral
                    && let Some(module_sym) =
                        self.resolve_module_file_symbol(&ext.expression.text())
                {
                    if let Some(export_eq) =
                        module_sym.exports.get(crate::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)
                    {

                        let entity_decl = export_eq
                            .declarations
                            .iter()
                            .find(|d| d.kind == SyntaxKind::ExportAssignment)
                            .cloned();
                        let scope_decl = module_sym
                            .declarations
                            .iter()
                            .find(|d| d.kind == SyntaxKind::ModuleDeclaration)
                            .cloned();
                        if let (Some(export_decl), Some(scope)) = (entity_decl, scope_decl)
                            && let crate::ast::NodeData::ExportAssignment(ea) = &export_decl.data
                            && ea.is_export_equals
                            && matches!(
                                ea.expression.kind,
                                SyntaxKind::Identifier | SyntaxKind::QualifiedName
                            )
                        {
                            self.push_scope(&scope);
                            let target = self.resolve_qualified_symbol(&ea.expression);
                            self.pop_scope();
                            if let Some(target) = target {
                                return target;
                            }
                        }
                    }
                    return module_sym;
                }

                if matches!(
                    data.module_reference.kind,
                    SyntaxKind::Identifier | SyntaxKind::QualifiedName
                ) {
                    let mut current = Arc::clone(&symbol);
                    for _ in 0..4 {
                        let next = current
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
                            Some(n) => current = n,
                            None => break,
                        }
                        if !current.flags.intersects(SymbolFlags::Alias) {
                            return current;
                        }
                    }
                    return current;
                }
            }
        }
        symbol
    }

    pub(crate) fn resolve_module_file_symbol(&self, specifier: &str) -> Option<Arc<Symbol>> {
        if !specifier.starts_with('.') {

            for file in self.program.source_files() {

                if file.external_module_indicator.is_some() {
                    continue;
                }
                if let crate::ast::NodeData::SourceFile(sf) = &file.node.data {
                    for stmt in sf.statements.iter() {
                        if let crate::ast::NodeData::ModuleDeclaration(md) = &stmt.data
                            && md.name.kind == SyntaxKind::StringLiteral
                            && md.name.text().trim_matches(['"', '\'']) == specifier
                        {
                            return self.program.symbol_map().symbol_of(stmt).cloned();
                        }
                    }
                }
            }
            return None;
        }
        let current = self.current_file.as_ref()?;
        let dir = match current.file_name.rfind('/') {
            Some(i) => &current.file_name[..i],
            None => "",
        };
        self.resolve_module_file_symbol_in(dir, specifier)
    }

    pub(crate) fn resolve_module_file_symbol_in(
        &self,
        dir: &str,
        specifier: &str,
    ) -> Option<Arc<Symbol>> {
        let stem = specifier.strip_prefix("./").unwrap_or(specifier);

        let stem = stem
            .strip_suffix(".js")
            .or_else(|| stem.strip_suffix(".jsx"))
            .unwrap_or(stem);
        let symbol_map = self.program.symbol_map();
        for cand in [
            format!("{dir}/{stem}.ts"),
            format!("{dir}/{stem}.tsx"),
            format!("{dir}/{stem}.d.ts"),
            format!("{dir}/{stem}/index.ts"),
            format!("{dir}/{stem}/index.d.ts"),
        ] {
            if let Some(sf) = self
                .program
                .source_files()
                .iter()
                .find(|f| f.file_name == cand)
            {
                if let Some(sym) = symbol_map.symbol_of(&sf.node) {
                    return Some(Arc::clone(sym));
                }
            }
        }
        None
    }


    pub(crate) fn for_each_module_statement(
        &self,
        module_symbol: &Arc<Symbol>,
        mut f: impl FnMut(&Arc<Node>) -> bool,
    ) {
        use crate::ast::NodeData;
        for decl in &module_symbol.declarations {
            let statements: Option<&Arc<crate::ast::NodeList>> = match &decl.data {
                NodeData::SourceFile(sf) => Some(&sf.statements),
                NodeData::ModuleDeclaration(md) => match &md.body {
                    Some(body) => match &body.data {
                        NodeData::ModuleBlock(b) => Some(&b.statements),
                        _ => None,
                    },
                    None => None,
                },
                _ => None,
            };
            if let Some(list) = statements {
                for s in list.iter() {
                    if f(s) {
                        return;
                    }
                }
            }
        }
    }


    pub(crate) fn class_name_text(class: &Arc<Node>) -> String {
        match &class.data {
            crate::ast::NodeData::ClassDeclaration(d) => {
                d.name.as_ref().map(|n| n.text().to_string()).unwrap_or_default()
            }
            crate::ast::NodeData::ClassExpression(d) => {
                d.name.as_ref().map(|n| n.text().to_string()).unwrap_or_default()
            }
            _ => String::new(),
        }
    }

    pub(crate) fn class_member_static_by_name(&self, class: &Arc<Node>, name: &str) -> Option<bool> {
        let members = match &class.data {
            crate::ast::NodeData::ClassDeclaration(d) => &d.members,
            crate::ast::NodeData::ClassExpression(d) => &d.members,
            _ => return None,
        };
        for member in members.iter() {
            let member_name = match &member.data {
                crate::ast::NodeData::PropertyDeclaration(d) => &d.name,
                crate::ast::NodeData::MethodDeclaration(d) => &d.name,
                crate::ast::NodeData::GetAccessorDeclaration(d) => &d.name,
                crate::ast::NodeData::SetAccessorDeclaration(d) => &d.name,
                _ => continue,
            };
            if member_name.kind == SyntaxKind::Identifier && member_name.text() == name {
                return Some(member.has_syntactic_modifier(ModifierFlags::Static));
            }
        }
        None
    }

    pub(crate) fn check_duplicate_function_implementations(&mut self, node: &Arc<Node>) {
        let crate::ast::NodeData::FunctionDeclaration(data) = &node.data else {
            return;
        };
        let Some(name) = &data.name else { return };
        if name.kind != SyntaxKind::Identifier {
            return;
        }
        let Some(parent) = node.parent.as_ref() else {
            return;
        };
        let stmts = match &parent.data {
            crate::ast::NodeData::SourceFile(sf) => Some(&sf.statements),
            crate::ast::NodeData::ModuleBlock(mb) => Some(&mb.statements),
            _ => None,
        };
        let Some(stmts) = stmts else {
            return;
        };
        let is_ambient = node.flags.contains(NodeFlags::Ambient)
            || self
                .current_file
                .as_ref()
                .is_some_and(|f| f.is_declaration_file);
        let fns: Vec<&Arc<Node>> = stmts
            .iter()
            .filter(|s| {
                s.kind == SyntaxKind::FunctionDeclaration
                    && matches!(&s.data, crate::ast::NodeData::FunctionDeclaration(d) if d
                        .name
                        .as_ref()
                        .is_some_and(|n| n.text() == name.text()))
            })
            .collect();

        if fns.first().is_none_or(|first| !Arc::ptr_eq(first, node)) {
            return;
        }
        let bodied = fns
            .iter()
            .filter(|f| {
                matches!(&f.data, crate::ast::NodeData::FunctionDeclaration(d) if d.body.is_some())
            })
            .count();
        let file = self.current_file.clone();
        if bodied >= 2 && !is_ambient {
            for f in &fns {
                if let crate::ast::NodeData::FunctionDeclaration(d) = &f.data
                    && let Some(fname) = &d.name
                {
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file.clone(),
                        fname.loc,
                        crate::diagnostics::messages_generated::DUPLICATE_FUNCTION_IMPLEMENTATION,
                        vec![],
                    ));
                }
            }
        }

        let is_ambient_decl = |f: &Arc<Node>| {
            f.has_syntactic_modifier(ModifierFlags::Ambient)
                || f.flags.contains(NodeFlags::Ambient)
        };
        let canonical = fns
            .iter()
            .find(|f| {
                matches!(&f.data, crate::ast::NodeData::FunctionDeclaration(d) if d.body.is_some())
            })
            .or_else(|| fns.first());
        if let Some(canonical) = canonical {
            let canonical_ambient = is_ambient_decl(canonical);
            for f in &fns {
                let has_body =
                    matches!(&f.data, crate::ast::NodeData::FunctionDeclaration(d) if d.body.is_some());
                if !has_body && is_ambient_decl(f) != canonical_ambient {
                    if let crate::ast::NodeData::FunctionDeclaration(d) = &f.data
                        && let Some(fname) = &d.name
                    {
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file.clone(),
                            fname.loc,
                            crate::diagnostics::messages_generated::
                                OVERLOAD_SIGNATURES_MUST_ALL_BE_AMBIENT_OR_NON_AMBIENT,
                            vec![],
                        ));
                    }
                }
            }
        }
    }

    pub(crate) fn check_overload_implementation_follows(&mut self, node: &Arc<Node>) {
        let crate::ast::NodeData::FunctionDeclaration(data) = &node.data else {
            return;
        };
        if data.body.is_some() {
            return;
        }
        let Some(name) = &data.name else { return };
        if name.kind != SyntaxKind::Identifier {
            return;
        }
        let Some(parent) = node.parent.as_ref() else {
            return;
        };
        let stmts = match &parent.data {
            crate::ast::NodeData::SourceFile(sf) => Some(&sf.statements),
            crate::ast::NodeData::ModuleBlock(mb) => Some(&mb.statements),
            _ => None,
        };
        let Some(stmts) = stmts else { return };
        let is_ambient = node.has_syntactic_modifier(ModifierFlags::Ambient)
            || node.flags.contains(NodeFlags::Ambient)
            || self.ambient_context_depth > 0

            || node
                .parent
                .as_ref()
                .is_some_and(|_| {
                    let mut anc = node.parent.as_ref();
                    let mut found = false;
                    while let Some(a) = anc {
                        if a.has_syntactic_modifier(ModifierFlags::Ambient) {
                            found = true;
                            break;
                        }
                        anc = a.parent.as_ref();
                    }
                    found
                })
            || self
                .current_file
                .as_ref()
                .is_some_and(|f| f.is_declaration_file);
        if is_ambient {
            return;
        }
        let next = stmts.iter().enumerate().find_map(|(i, s)| {
            if Arc::ptr_eq(s, node) {
                stmts.nodes.get(i + 1).cloned()
            } else {
                None
            }
        });

        if next.as_ref().is_some_and(|n| {
            matches!(&n.data, crate::ast::NodeData::FunctionDeclaration(d) if d
                .name
                .as_ref()
                .is_some_and(|n2| n2.text() == name.text()))
        }) {
            return;
        }

        if let Some(n) = &next
            && matches!(&n.data, crate::ast::NodeData::FunctionDeclaration(d) if d.body.is_some())
            && n.kind == SyntaxKind::FunctionDeclaration
        {
            if let crate::ast::NodeData::FunctionDeclaration(d) = &n.data
                && let Some(next_name) = &d.name
                && next_name.kind == SyntaxKind::Identifier
                && next_name.text() != name.text()
            {

                let already = self
                    .diagnostics
                    .get_all()
                    .iter()
                    .any(|d| d.code == 2389 && d.loc == next_name.loc);
                if !already {
                    let file = self.current_file.clone();
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        next_name.loc,
                        crate::diagnostics::messages_generated::
                            FUNCTION_IMPLEMENTATION_NAME_MUST_BE_0,
                        vec![name.text().to_string()],
                    ));
                }
                return;
            }
        }

        let already = self
            .diagnostics
            .get_all()
            .iter()
            .any(|d| d.code == 2391 && d.loc == name.loc);
        if !already {
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                name.loc,
                crate::diagnostics::messages_generated::
                    FUNCTION_IMPLEMENTATION_IS_MISSING_OR_NOT_IMMEDIATELY_FOLLOWING_THE_DECLARATION,
                vec![],
            ));
        }
    }

    pub(crate) fn check_multiple_constructor_implementations(&mut self, node: &Arc<Node>) {
        let Some(class) = node.parent.as_ref() else {
            return;
        };
        if !matches!(class.kind, SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression) {
            return;
        }
        let crate::ast::NodeData::ClassDeclaration(cd) = &class.data else {
            return;
        };
        let ctors: Vec<&Arc<Node>> = cd
            .members
            .iter()
            .filter(|m| m.kind == SyntaxKind::Constructor)
            .collect();
        if ctors.first().is_none_or(|first| !Arc::ptr_eq(first, node)) {
            return;
        }
        let bodied = ctors
            .iter()
            .filter(|c| {
                matches!(&c.data, crate::ast::NodeData::ConstructorDeclaration(d) if d.body.is_some())
            })
            .count();
        if bodied < 2 {
            return;
        }
        let file = self.current_file.clone();
        for ctor in ctors {
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file.clone(),
                ctor.loc,
                crate::diagnostics::messages_generated::
                    MULTIPLE_CONSTRUCTOR_IMPLEMENTATIONS_ARE_NOT_ALLOWED,
                vec![],
            ));
        }
    }

    pub(crate) fn check_invalid_initializer_reference(&mut self, node: &Arc<Node>, name: &str) -> bool {
        if self.emit_standard_class_fields {
            return false;
        }
        let Some(parent) = node.parent.as_ref() else {
            return false;
        };
        let Some(property) = crate::ast::utilities::find_ancestor(parent, |n| {
            n.kind == SyntaxKind::PropertyDeclaration
        }) else {
            return false;
        };

        if let Some(sym) = self.resolve_identifier(node) {
            let binds_in_initializer_fn = sym.declarations.iter().any(|d| {
                let mut cur = d.parent.as_ref();
                while let Some(a) = cur {
                    if Arc::ptr_eq(a, &property) {
                        return false;
                    }
                    if matches!(
                        a.kind,
                        SyntaxKind::FunctionDeclaration
                            | SyntaxKind::FunctionExpression
                            | SyntaxKind::ArrowFunction
                    ) {
                        return true;
                    }
                    cur = a.parent.as_ref();
                }
                false
            });
            if binds_in_initializer_fn {
                return false;
            }
        }
        if property.has_syntactic_modifier(ModifierFlags::Static) {
            return false;
        }
        let Some(class) = property.parent.as_ref() else {
            return false;
        };
        if !matches!(class.kind, SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression) {
            return false;
        }

        let crate::ast::NodeData::ClassDeclaration(cd) = &class.data else {
            return false;
        };
        let ctor = cd.members.iter().find(|m| {
            m.kind == SyntaxKind::Constructor
                && matches!(&m.data, crate::ast::NodeData::ConstructorDeclaration(d) if d.body.is_some())
        });
        let Some(ctor) = ctor else {
            return false;
        };
        let symbol_map = self.program.symbol_map();
        let ctor_has_name = symbol_map
            .locals
            .get(&ctor.id())
            .is_some_and(|locals| {
                locals
                    .get(name)
                    .is_some_and(|sym| sym.flags.intersects(SymbolFlags::VALUE))
            });
        if !ctor_has_name {
            return false;
        }
        let file = self.current_file.clone();
        let property_name = property
            .name()
            .map(|n| n.text().to_string())
            .unwrap_or_default();
        self.diagnostics.add(crate::ast::Diagnostic::new(
            file,
            node.loc,
            crate::diagnostics::messages_generated::
                INITIALIZER_OF_INSTANCE_MEMBER_VARIABLE_0_CANNOT_REFERENCE_IDENTIFIER_1_DECLARED_IN_THE_CONSTRUCTOR,
            vec![property_name, name.to_string()],
        ));
        true
    }
}
