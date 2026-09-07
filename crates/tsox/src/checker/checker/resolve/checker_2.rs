#![allow(unused_imports)]

use super::*;

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

                    if let Some(parent) = &merged.parent {
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
            if let crate::ast::NodeData::ElementAccessExpression(data) = &expression.data {
                if let crate::ast::NodeData::StringLiteral(key) = &data.argument_expression.data {
                    return Some(key.text.clone());
                }

                if let crate::ast::NodeData::NumericLiteral(key) = &data.argument_expression.data {
                    return Some(key.text.clone());
                }

                if let crate::ast::NodeData::Identifier(key) = &data.argument_expression.data {
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
                        if is_type_only_node(n) {
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
