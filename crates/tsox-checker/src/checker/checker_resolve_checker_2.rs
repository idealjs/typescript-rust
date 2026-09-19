#![allow(unused_imports)]

use crate::checker::checker_resolve::*;

impl Checker {
    pub(crate) fn get_referenced_value_symbol(
        &self,
        node: &Node,
        start_in_declaration_container: bool,
    ) -> Option<Arc<Symbol>> {
        let symbol_map = self.program.symbol_map();

        if let Some(sym) = symbol_map.symbol_of(node) {
            return Some(Arc::clone(sym));
        }

        let location = if start_in_declaration_container {
            node
        } else {
            node
        };

        let meaning = SymbolFlags::ExportValue
            .union(SymbolFlags::VALUE)
            .union(SymbolFlags::Alias);
        self.resolve_identifier_at_location(location, node_name(node)?, meaning)
    }

    #[allow(dead_code)]
    pub(crate) fn find_parent_declaration_container(&self, _node: &Node) -> Option<u64> {
        for &container_id in self.scope_stack.iter().rev() {
            let symbol_map = self.program.symbol_map();
            if let Some(container_sym) = symbol_map.symbols.get(&container_id) {
                if container_sym
                    .flags
                    .intersects(SymbolFlags::MODULE | SymbolFlags::ENUM)
                {
                    return Some(container_id);
                }
            }
        }
        None
    }

    pub fn get_referenced_export_container(&self, node: &Node, prefix_locals: bool) -> Option<u64> {
        let start_in_declaration_container = is_module_or_enum_name(node);
        if let Some(symbol) = self.get_referenced_value_symbol(node, start_in_declaration_container)
        {
            if symbol.flags.intersects(SymbolFlags::ExportValue) {
                if let Some(ref export_symbol) = symbol.export_symbol {
                    let merged = self.get_merged_symbol(export_symbol);
                    if !prefix_locals
                        && merged.flags.intersects(SymbolFlags::EXPORT_HAS_LOCAL)
                        && !merged.flags.intersects(SymbolFlags::VARIABLE)
                    {
                        return None;
                    }

                    if let Some(parent) = &merged.parent() {
                        if parent.flags.intersects(SymbolFlags::ValueModule)
                            && parent.value_declaration.is_some()
                        {
                            return Some(parent.value_declaration.as_ref().unwrap().id());
                        }

                        for &container_id in self.scope_stack.iter().rev() {
                            let symbol_map = self.program.symbol_map();
                            if let Some(container_sym) = symbol_map.symbols.get(&container_id) {
                                if Arc::ptr_eq(container_sym, parent) {
                                    return Some(container_id);
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    pub fn get_referenced_import_declaration(&self, node: &Node) -> Option<Arc<Node>> {
        if let Some(symbol) = self.get_referenced_value_symbol(node, false) {
            if is_non_local_alias(&symbol, SymbolFlags::VALUE)
                && !self.is_type_only_alias_declaration(&symbol)
            {
                return self.get_declaration_of_alias_symbol(&symbol);
            }
        }
        None
    }

    pub fn get_referenced_value_declaration(&self, node: &Node) -> Option<Arc<Node>> {
        if let Some(symbol) = self.get_referenced_value_symbol(node, false) {
            let export_sym = self.get_export_symbol_of_value_symbol_if_exported(&symbol);
            return export_sym.value_declaration.clone();
        }
        None
    }

    pub fn get_referenced_value_declarations(&self, node: &Node) -> Vec<Arc<Node>> {
        let mut declarations = Vec::new();
        if let Some(symbol) = self.get_referenced_value_symbol(node, false) {
            let export_sym = self.get_export_symbol_of_value_symbol_if_exported(&symbol);
            for decl in export_sym.declarations.iter() {
                match decl.kind {
                    SyntaxKind::VariableDeclaration
                    | SyntaxKind::Parameter
                    | SyntaxKind::BindingElement
                    | SyntaxKind::PropertyDeclaration
                    | SyntaxKind::PropertyAssignment
                    | SyntaxKind::ShorthandPropertyAssignment
                    | SyntaxKind::EnumMember
                    | SyntaxKind::ObjectLiteralExpression
                    | SyntaxKind::FunctionDeclaration
                    | SyntaxKind::FunctionExpression
                    | SyntaxKind::ArrowFunction
                    | SyntaxKind::ClassDeclaration
                    | SyntaxKind::ClassExpression
                    | SyntaxKind::EnumDeclaration
                    | SyntaxKind::MethodDeclaration
                    | SyntaxKind::GetAccessor
                    | SyntaxKind::SetAccessor
                    | SyntaxKind::ModuleDeclaration => {
                        declarations.push(Arc::clone(decl));
                    }
                    _ => {}
                }
            }
        }
        declarations
    }

    pub fn get_element_access_expression_name(&self, expression: &Node) -> Option<String> {
        if expression.kind == SyntaxKind::ElementAccessExpression {
            if let tsox_frontend::ast::NodeData::ElementAccessExpression(data) = &expression.data {
                if let tsox_frontend::ast::NodeData::StringLiteral(key) =
                    &data.argument_expression.data
                {
                    return Some(key.text.clone());
                }

                if let tsox_frontend::ast::NodeData::NumericLiteral(key) =
                    &data.argument_expression.data
                {
                    return Some(key.text.clone());
                }

                if let tsox_frontend::ast::NodeData::Identifier(key) =
                    &data.argument_expression.data
                {
                    return Some(key.text.clone());
                }
            }
        }
        None
    }

    pub fn get_referenced_member_value_declaration(&self, node: &Node) -> Option<Arc<Node>> {
        let symbol_map = self.program.symbol_map();
        let s = symbol_map.symbol_of(node).map(|s| Arc::clone(s));
        if s.is_none() {
            if let Some(sym) = symbol_map.symbol_of(node) {
                let merged = self.get_merged_symbol(sym);
                let export_sym = self.get_export_symbol_of_value_symbol_if_exported(&merged);
                return export_sym.value_declaration.clone();
            }
        }
        if let Some(ref s) = s {
            let export_sym = self.get_export_symbol_of_value_symbol_if_exported(s);
            return export_sym.value_declaration.clone();
        }
        None
    }

    pub fn get_merged_symbol(&self, symbol: &Arc<Symbol>) -> Arc<Symbol> {
        if let Some(_target_id) = self.merged_symbols.get(&symbol.id()) {}
        Arc::clone(symbol)
    }

    pub(crate) fn get_export_symbol_of_value_symbol_if_exported(
        &self,
        symbol: &Arc<Symbol>,
    ) -> Arc<Symbol> {
        let mut result = Arc::clone(symbol);
        if symbol.flags.intersects(SymbolFlags::ExportValue) {
            if let Some(ref export_sym) = symbol.export_symbol {
                result = self.get_merged_symbol(export_sym);
            }
        }
        result
    }

    /// Go getTypeOnlyAliasDeclarationEx(Value)：沿别名链回溯，链上任一环的
    /// 声明是 type-only 即返回该声明与是否 export 形态
    pub(crate) fn type_only_alias_value_declaration(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Option<(Arc<Node>, bool)> {
        self.type_only_alias_value_declaration_impl(symbol)
    }

    fn type_only_alias_value_declaration_impl(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Option<(Arc<Node>, bool)> {
        let mut current = Arc::clone(symbol);
        for hop in 0..16 {
            if !current.flags.contains(SymbolFlags::Alias)
                || current.flags.contains(SymbolFlags::VALUE)
            {
                if hop == 0 {
                    return None;
                }
                break;
            }
            if self.is_type_only_alias_declaration(&current) {
                let decl = self.get_declaration_of_alias_symbol(&current)?;
                let is_export_form = matches!(
                    decl.kind,
                    SyntaxKind::ExportSpecifier | SyntaxKind::ExportDeclaration
                );
                return Some((decl, is_export_form));
            }
            let next = self.resolve_alias_base(Arc::clone(&current));
            if Arc::ptr_eq(&next, &current) {
                break;
            }
            current = next;
        }
        // 符号链塌缩（export * 中转直接落到目标）时走模块导出链回溯
        if let Some(decl) = self.get_declaration_of_alias_symbol(symbol) {
            let imported_name = match &decl.data {
                tsox_frontend::ast::NodeData::ImportSpecifier(d) => d
                    .property_name
                    .as_ref()
                    .map_or_else(|| d.name.text().to_string(), |p| p.text().to_string()),
                tsox_frontend::ast::NodeData::ImportClause(d) => d
                    .name
                    .as_ref()
                    .map(|n| n.text().to_string())
                    .unwrap_or_default(),
                _ => return None,
            };
            let mut import_decl = decl.parent();
            while let Some(nd) = &import_decl {
                if matches!(
                    nd.data,
                    tsox_frontend::ast::NodeData::ImportDeclaration(_)
                ) {
                    break;
                }
                import_decl = nd.parent();
            }
            let Some(import_decl) = import_decl.clone() else {
                return None;
            };
            let tsox_frontend::ast::NodeData::ImportDeclaration(id) = &import_decl.data
            else {
                return None;
            };
            let spec = id
                .module_specifier
                .text()
                .trim_matches(['"', '\'', '`'])
                .to_string();
            let module_sym = self.resolve_module_file_symbol(&spec);
            let Some(module_sym) = module_sym else {
                return None;
            };
            if let Some(decl) =
                self.find_type_only_export_in_module_chain(&module_sym, &imported_name, 8)
            {
                let is_export_form = matches!(
                    decl.kind,
                    SyntaxKind::ExportSpecifier | SyntaxKind::ExportDeclaration
                );
                return Some((decl, is_export_form));
            }
        }
        None
    }

    /// 模块导出链上按名回溯：任一环是 type-only 导出即返回该声明
    /// （覆盖 `export type {}`、`export * from` 中转）
    pub(crate) fn find_type_only_export_in_module_chain(
        &mut self,
        module_sym: &Arc<Symbol>,
        name: &str,
        depth: usize,
    ) -> Option<Arc<Node>> {
        if depth == 0 {
            return None;
        }
        let mut hits: Vec<(String, Option<String>, Option<Arc<Node>>)> = Vec::new();
        let mut star_hits: Vec<(String, Option<String>)> = Vec::new();
        let mut value_star_hits: Vec<(String, Option<String>)> = Vec::new();
        let mut star_type_only: Option<Arc<Node>> = None;
        let mut type_only_hit: Option<Arc<Node>> = None;
        self.for_each_module_statement(module_sym, |stmt| {
            if let tsox_frontend::ast::NodeData::ExportDeclaration(d) = &stmt.data {
                if let Some(clause) = &d.export_clause
                    && let tsox_frontend::ast::NodeData::NamedExports(ne) = &clause.data
                {
                    for el in ne.elements.iter() {
                        if let tsox_frontend::ast::NodeData::ExportSpecifier(spec) = &el.data
                            && spec.name.text().trim_matches(['"', '\'', '`']) == name
                        {
                            let imported = spec
                                .property_name
                                .as_ref()
                                .unwrap_or(&spec.name)
                                .text()
                                .trim_matches(['"', '\'', '`'])
                                .to_string();
                            let module_text = d.module_specifier.as_ref().map(|m| {
                                m.text().trim_matches(['"', '\'', '`']).to_string()
                            });
                            if d.is_type_only || spec.is_type_only {
                                type_only_hit = Some(Arc::clone(el));
                                return true;
                            }
                            hits.push((
                                imported,
                                module_text,
                                Some(Arc::clone(el)),
                            ));
                            return true;
                        }
                    }
                } else if let Some(clause) = &d.export_clause
                    && clause.kind == SyntaxKind::NamespaceExport
                    && d.is_type_only
                    && tsox_frontend::ast::node_data_generated::node_name(clause)
                        .is_some_and(|n| n.text() == name)
                {
                    // export * as ns（type-only 形态报 TS1362）
                    type_only_hit = Some(Arc::clone(stmt));
                } else if d.export_clause.is_none() {
                    // export * from '...'（具名导出优先，星号仅兜底：
                    // 先收集，named 命中后跳过）
                    let module_text = d.module_specifier.as_ref().map(|m| {
                        m.text().trim_matches(['"', '\'', '`']).to_string()
                    });
                    if d.is_type_only {
                        star_type_only = Some(Arc::clone(stmt));
                    } else {
                        value_star_hits.push((name.to_string(), module_text.clone()));
                    }
                    star_hits.push((name.to_string(), module_text));
                }
            }
            false
        });
        if let Some(decl) = type_only_hit {
            return Some(decl);
        }
        // 具名导出未命中时星号兜底；export type * 命中即 type-only
        //（同模块并存值星号时值侧遮蔽：名字经值星号可达则不算 type-only）
        if hits.is_empty() {
            // export type * 与值星号并存：名字经值星号以「值意义」可达则遮蔽
            if let Some(decl) = star_type_only {
                let mut shadowed = false;
                // 本地导出（export class C 等隐式导出）以值意义遮蔽星号
                if let Some(local) = self.namespace_member_recursive(module_sym, name)
                    && local.flags.intersects(SymbolFlags::VALUE.union(SymbolFlags::Class))
                {
                    shadowed = true;
                }
                for (imported, module_text) in value_star_hits.clone() {
                    let Some(text) = module_text else {
                        continue;
                    };
                    let target_module = self
                        .resolve_module_spec_from(module_sym, &text)
                        .or_else(|| self.resolve_module_file_symbol(&text));
                    let Some(target_module) = target_module else {
                        continue;
                    };
                    if let Some(sym) =
                        self.resolve_module_member_symbol(&target_module, &imported, 6)
                        && sym.flags.intersects(
                            SymbolFlags::VALUE.union(SymbolFlags::Class),
                        )
                    {
                        shadowed = true;
                        break;
                    }
                }
                if !shadowed {
                    return Some(decl);
                }
            }
            for (imported, module_text) in star_hits {
                let Some(text) = module_text else {
                    continue;
                };
                let Some(target_module) =
                    self.resolve_module_spec_from(module_sym, &text)
                else {
                    continue;
                };
                if let Some(found) = self.find_type_only_export_in_module_chain(
                    &target_module,
                    &imported,
                    depth - 1,
                ) {
                    return Some(found);
                }
            }
        }
        for (imported, module_text, hit_spec) in hits {
            let Some(text) = module_text else {
                // 本地具名导出 export { a }：回溯模块本地符号是否 import-type
                if hit_spec.is_some() {
                    if let Some(local) = self
                        .namespace_member_recursive(module_sym, &imported)
                        .or_else(|| module_sym.exports.get(&imported).cloned())
                    {
                        for d in &local.declarations {
                            let type_only = match &d.data {
                                tsox_frontend::ast::NodeData::ImportSpecifier(sd) => {
                                    sd.is_type_only || import_clause_phase_is_type(d)
                                }
                                tsox_frontend::ast::NodeData::ImportClause(cd) => {
                                    cd.phase_modifier == Some(SyntaxKind::TypeKeyword)
                                }
                                _ => false,
                            };
                            if type_only {
                                return Some(Arc::clone(d));
                            }
                            if matches!(
                                &d.data,
                                tsox_frontend::ast::NodeData::ImportSpecifier(_)
                                    | tsox_frontend::ast::NodeData::ImportClause(_)
                            ) {
                                break;
                            }
                        }
                    }
                }
                continue;
            };
            let Some(target_module) =
                self.resolve_module_spec_from(module_sym, &text)
            else {
                continue;
            };
            if let Some(found) = self.find_type_only_export_in_module_chain(
                &target_module,
                &imported,
                depth - 1,
            ) {
                return Some(found);
            }
        }
        None
    }

    pub(crate) fn is_type_only_alias_declaration(&self, symbol: &Arc<Symbol>) -> bool {
        if let Some(node) = self.get_declaration_of_alias_symbol(symbol) {
            let current = Some(Arc::clone(&node));
            while let Some(ref n) = current {
                match n.kind {
                    SyntaxKind::ImportEqualsDeclaration | SyntaxKind::ExportDeclaration => {
                        return is_type_only_node(n);
                    }
                    SyntaxKind::ImportClause
                    | SyntaxKind::ImportSpecifier
                    | SyntaxKind::ExportSpecifier => {
                        let mut anc = n.parent();
                        let mut anc_type_only = false;
                        while let Some(p) = anc {
                            match p.kind {
                                SyntaxKind::ImportDeclaration
                                | SyntaxKind::ExportDeclaration => {
                                    anc_type_only = is_type_only_node(&p);
                                    break;
                                }
                                SyntaxKind::NamedImports
                                | SyntaxKind::NamedExports
                                | SyntaxKind::NamespaceImport => {
                                    anc = p.parent();
                                }
                                _ => break,
                            }
                        }
                        if is_type_only_node(n) || anc_type_only {
                            return true;
                        }

                        break;
                    }
                    _ => break,
                }
            }
        }
        false
    }

    pub(crate) fn get_declaration_of_alias_symbol(
        &self,
        symbol: &Arc<Symbol>,
    ) -> Option<Arc<Node>> {
        symbol
            .declarations
            .iter()
            .filter(|d| is_alias_symbol_declaration(d))
            .last()
            .cloned()
    }

    pub(crate) fn resolve_identifier_at_location(
        &self,
        _location: &Node,
        name: &str,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        let symbol_map = self.program.symbol_map();

        for &container_id in self.scope_stack.iter().rev() {
            if let Some(locals) = symbol_map.locals.get(&container_id) {
                if let Some(sym) = locals.get(name) {
                    if sym.flags.intersects(meaning) {
                        return self.follow_alias(sym);
                    }
                }
            }

            if let Some(container_sym) = symbol_map.symbols.get(&container_id) {
                if let Some(sym) = container_sym.members.get(name) {
                    if sym.flags.intersects(meaning) {
                        return self.follow_alias(sym);
                    }
                }

                if container_sym.flags.intersects(SymbolFlags::MODULE) {
                    if let Some(sym) = container_sym.exports.get(name) {
                        let is_export_specifier = sym.flags == SymbolFlags::Alias
                            && sym
                                .declarations
                                .iter()
                                .any(|d| d.kind == SyntaxKind::ExportSpecifier);
                        if !is_export_specifier {
                            return self.follow_alias(sym);
                        }
                    }
                }

                if container_sym.flags.intersects(SymbolFlags::ENUM) {
                    if let Some(sym) = container_sym.exports.get(name) {
                        if sym.flags.intersects(meaning) {
                            return self.follow_alias(sym);
                        }
                    }
                }
            }
        }

        if let Some(sym) = self.globals.get(name) {
            if sym
                .flags
                .intersects(meaning.union(SymbolFlags::GlobalLookup))
            {
                return Some(Arc::clone(sym));
            }
        }

        None
    }
}

/// ImportSpecifier 所属 ImportClause 的 phase 是 `import type` 形态
fn import_clause_phase_is_type(specifier: &Arc<Node>) -> bool {
    specifier
        .parent()
        .and_then(|named| named.parent())
        .and_then(|clause| {
            if let tsox_frontend::ast::NodeData::ImportClause(cd) = &clause.data {
                Some(cd.phase_modifier == Some(SyntaxKind::TypeKeyword))
            } else {
                None
            }
        })
        .unwrap_or(false)
}
