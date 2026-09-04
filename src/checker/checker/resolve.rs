use std::sync::Arc;

use crate::ast::{
    Node, Symbol, SymbolFlags, SymbolTable, SyntaxKind,
};


use super::*;

impl Checker {
    pub fn resolve_identifier(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        self.resolve_identifier_with_meaning(node, SymbolFlags::all())
    }

    pub fn resolve_identifier_with_meaning(
        &self,
        node: &Arc<Node>,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        let result = self.resolve_identifier_with_meaning_inner(node, meaning);

        if let Some(sym) = &result {
            let mut bits = meaning;
            if meaning.intersects(SymbolFlags::VALUE) {
                bits |= SymbolFlags::FunctionScopedVariable
                    | SymbolFlags::BlockScopedVariable;
            }
            self.record_symbol_reference(sym, bits);
        }
        result
    }

    fn record_symbol_reference(&self, symbol: &Arc<Symbol>, bits: SymbolFlags) {
        self.symbol_reference_kinds
            .entry(symbol.id())
            .and_modify(|f| *f |= bits)
            .or_insert(bits);
    }

    fn alias_chain_hits_meaning(&self, sym: &Arc<Symbol>, meaning: SymbolFlags) -> bool {
        if !sym.flags.intersects(SymbolFlags::Alias) {
            return false;
        }
        match self.follow_alias(sym) {
            Some(target) if !Arc::ptr_eq(&target, sym) => target.flags.intersects(meaning),
            _ => true,
        }
    }

    fn resolve_identifier_with_meaning_inner(
        &self,
        node: &Arc<Node>,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        let name = match &node.data {
            crate::ast::NodeData::Identifier(data) => data.text.as_str(),
            _ => return None,
        };
        let symbol_map = self.program.symbol_map();

        for &container_id in self.scope_stack.iter().rev() {

            if let Some(locals) = symbol_map.locals.get(&container_id) {
                if let Some(sym) = locals.get(name) {
                    if sym.flags.intersects(meaning) || self.alias_chain_hits_meaning(&sym, meaning) {
                        return self.follow_alias(sym);
                    }
                }
            }

            if let Some(container_sym) = symbol_map.symbols.get(&container_id) {

                if !container_sym.flags.intersects(SymbolFlags::Class)
                    || container_sym.flags.intersects(SymbolFlags::Function)
                {
                    if let Some(sym) = container_sym.members.get(name) {
                        if sym.flags.intersects(meaning) || self.alias_chain_hits_meaning(&sym, meaning) {
                            return self.follow_alias(sym);
                        }
                    }
                }

                if container_sym.flags.intersects(SymbolFlags::MODULE)
                    && !container_sym.flags.intersects(SymbolFlags::Class)
                {
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

                    if let Some(merged) = self.globals.get(container_sym.name.as_str()) {
                        if !Arc::ptr_eq(merged, container_sym)
                            && merged.flags.intersects(SymbolFlags::MODULE)
                        {
                            if let Some(sym) = merged.exports.get(name) {
                                if sym.flags.intersects(meaning) || self.alias_chain_hits_meaning(&sym, meaning) {
                                    return self.follow_alias(sym);
                                }
                            }
                            if let Some(sym) = self.ambient_namespace_local(merged, name) {
                                if sym.flags.intersects(meaning) || self.alias_chain_hits_meaning(&sym, meaning) {
                                    return self.follow_alias(&sym);
                                }
                            }
                        }
                    }
                }

                if container_sym.flags.intersects(SymbolFlags::ENUM) {
                    if let Some(sym) = container_sym.exports.get(name) {
                        if sym.flags.intersects(meaning) || self.alias_chain_hits_meaning(&sym, meaning) {
                            return self.follow_alias(sym);
                        }
                    }
                }

                if let Some(sym) = container_sym.members.get(name) {
                    if sym.flags.intersects(meaning & SymbolFlags::TYPE) || self.alias_chain_hits_meaning(&sym, meaning) {
                        return self.follow_alias(sym);
                    }
                }
            }
        }

        {

            const ANCESTRY_CONTAINERS: &[SyntaxKind] = &[
                SyntaxKind::SourceFile,
                SyntaxKind::ModuleDeclaration,
                SyntaxKind::Block,
                SyntaxKind::CatchClause,
                SyntaxKind::ForStatement,
                SyntaxKind::ForInStatement,
                SyntaxKind::ForOfStatement,
                SyntaxKind::FunctionDeclaration,
                SyntaxKind::FunctionExpression,
                SyntaxKind::ArrowFunction,
                SyntaxKind::MethodDeclaration,

                SyntaxKind::MethodSignature,

                SyntaxKind::CallSignature,
                SyntaxKind::ConstructSignature,
                SyntaxKind::FunctionType,
                SyntaxKind::ConstructorType,
                SyntaxKind::MappedType,
                SyntaxKind::Constructor,
                SyntaxKind::GetAccessor,
                SyntaxKind::SetAccessor,
                SyntaxKind::InterfaceDeclaration,
                SyntaxKind::ClassDeclaration,
                SyntaxKind::ClassExpression,
                SyntaxKind::TypeAliasDeclaration,
                SyntaxKind::EnumDeclaration,
            ];
            let mut ancestor = node.parent.as_ref();
            while let Some(a) = ancestor {
                if !ANCESTRY_CONTAINERS.contains(&a.kind) {
                    ancestor = a.parent.as_ref();
                    continue;
                }
                let aid = a.id();
                if let Some(locals) = symbol_map.locals.get(&aid) {
                    if let Some(sym) = locals.get(name)
                        && (sym.flags.intersects(meaning) || self.alias_chain_hits_meaning(&sym, meaning))
                    {
                        return self.follow_alias(sym);
                    }
                }
                if let Some(a_sym) = symbol_map.symbols.get(&aid) {
                    if !a_sym.flags.intersects(SymbolFlags::Class) {
                        if let Some(sym) = a_sym.members.get(name)
                            && (sym.flags.intersects(meaning) || self.alias_chain_hits_meaning(&sym, meaning))
                        {
                            return self.follow_alias(sym);
                        }
                        if a_sym.flags.intersects(SymbolFlags::MODULE | SymbolFlags::ENUM)
                            && let Some(sym) = a_sym.exports.get(name)
                            && (sym.flags.intersects(meaning) || self.alias_chain_hits_meaning(&sym, meaning))
                        {
                            return self.follow_alias(sym);
                        }

                        if a_sym.flags.intersects(SymbolFlags::MODULE) {
                            if let Some(merged) = self.globals.get(a_sym.name.as_str()) {
                                if !Arc::ptr_eq(merged, a_sym)
                                    && merged.flags.intersects(SymbolFlags::MODULE)
                                {
                                    if let Some(sym) = merged.exports.get(name)
                                        && (sym.flags.intersects(meaning) || self.alias_chain_hits_meaning(&sym, meaning))
                                    {
                                        return self.follow_alias(sym);
                                    }
                                    if let Some(sym) =
                                    self.ambient_namespace_local(merged, name)
                                    && (sym.flags.intersects(meaning) || self.alias_chain_hits_meaning(&sym, meaning))
                                {
                                    return self.follow_alias(&sym);
                                }
                            }
                        }
                    }
                    }

                    if let Some(sym) = a_sym.members.get(name)
                        && (sym.flags.intersects(meaning & SymbolFlags::TYPE) || self.alias_chain_hits_meaning(&sym, meaning))
                    {
                        return self.follow_alias(sym);
                    }
                }
                ancestor = a.parent.as_ref();
            }
        }

        if self.function_scope_count > 0
            && name == "arguments"
            && meaning.intersects(SymbolFlags::VARIABLE)
        {
            if let Some(ref sym) = self.arguments_symbol {
                return Some(Arc::clone(sym));
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

    pub fn follow_alias(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {

        if symbol.flags.intersects(SymbolFlags::Alias) {
            self.record_symbol_reference(
                symbol,
                SymbolFlags::VALUE
                    | SymbolFlags::TYPE
                    | SymbolFlags::NAMESPACE
                    | SymbolFlags::FunctionScopedVariable
                    | SymbolFlags::BlockScopedVariable,
            );
        }
        if !symbol.flags.intersects(SymbolFlags::Alias) {
            return Some(Arc::clone(symbol));
        }

        let is_pure_alias = symbol.flags == SymbolFlags::Alias
            || (symbol.flags.intersects(SymbolFlags::Alias)
                && symbol.flags.intersects(SymbolFlags::Assignment));
        if !is_pure_alias {
            return Some(Arc::clone(symbol));
        }

        let mut current = Arc::clone(symbol);
        let mut seen: Vec<*const Symbol> = vec![Arc::as_ptr(symbol)];
        loop {
            if let Some(ref target) = current.export_symbol {
                let target_ptr = Arc::as_ptr(target);
                if seen.contains(&target_ptr) {

                    return Some(Arc::clone(&current));
                }
                let is_pure = target.flags == SymbolFlags::Alias
                    || (target.flags.intersects(SymbolFlags::Alias)
                        && target.flags.intersects(SymbolFlags::Assignment));
                if is_pure {
                    seen.push(target_ptr);
                    current = Arc::clone(target);
                    continue;
                }
                return Some(Arc::clone(target));
            } else {

                return Some(Arc::clone(&current));
            }
        }
    }

    fn get_referenced_value_symbol(
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
    fn find_parent_declaration_container(&self, _node: &Node) -> Option<u64> {

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
        if let Some(_target_id) = self.merged_symbols.get(&symbol.id()) {

        }
        Arc::clone(symbol)
    }

    fn get_export_symbol_of_value_symbol_if_exported(&self, symbol: &Arc<Symbol>) -> Arc<Symbol> {
        let mut result = Arc::clone(symbol);
        if symbol.flags.intersects(SymbolFlags::ExportValue) {
            if let Some(ref export_sym) = symbol.export_symbol {
                result = self.get_merged_symbol(export_sym);
            }
        }
        result
    }

    fn is_type_only_alias_declaration(&self, symbol: &Arc<Symbol>) -> bool {
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

    fn get_declaration_of_alias_symbol(&self, symbol: &Arc<Symbol>) -> Option<Arc<Node>> {

        symbol
            .declarations
            .iter()
            .filter(|d| is_alias_symbol_declaration(d))
            .last()
            .cloned()
    }

    fn resolve_identifier_at_location(
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

    pub fn merge_symbol_table(
        &mut self,
        target: &mut SymbolTable,
        source: &SymbolTable,
        unidirectional: bool,
        merged_parent: Option<u64>,
    ) {

        let entries: Vec<(String, Arc<Symbol>)> = source
            .iter()
            .map(|(k, v)| (k.clone(), Arc::clone(v)))
            .collect();
        for (name, source_symbol) in entries {
            if let Some(target_symbol) = target.entries.get_mut(&name) {

                let merged = self.merge_symbol(target_symbol, &source_symbol, unidirectional);
                let is_transient = merged.flags.intersects(SymbolFlags::Transient);
                *target_symbol = merged;
                if let Some(_parent_id) = merged_parent {
                    if is_transient {

                    }
                }
            } else {

                let merged = self.get_merged_symbol(&source_symbol);
                target.insert(name, merged);
            }
        }
    }

    pub fn merge_symbol(
        &mut self,
        target: &Arc<Symbol>,
        source: &Arc<Symbol>,
        unidirectional: bool,
    ) -> Arc<Symbol> {
        let excluded = get_excluded_symbol_flags(source.flags);
        if target.flags.intersects(excluded) == false
            || (source.flags | target.flags).intersects(SymbolFlags::Assignment)
        {
            if Arc::ptr_eq(target, source) {
                return Arc::clone(target);
            }

            let effective_target = if !target.flags.intersects(SymbolFlags::Transient) {
                let resolved_target = self.resolve_symbol(target);
                if resolved_target
                    .flags
                    .intersects(get_excluded_symbol_flags(source.flags))
                    == false
                    || (source.flags | resolved_target.flags).intersects(SymbolFlags::Assignment)
                {
                    if let Some(cloned) = self.clone_symbol(&resolved_target) {
                        cloned
                    } else {
                        return Arc::clone(source);
                    }
                } else {

                    return Arc::clone(source);
                }
            } else {
                Arc::clone(target)
            };

            let mut source_flags = source.flags;
            if !effective_target
                .flags
                .intersects(SymbolFlags::ConstEnumOnlyModule)
            {
                source_flags.remove(SymbolFlags::ConstEnumOnlyModule);
            }
            let merged_flags = effective_target.flags | source_flags;

            let mut merged = Symbol::new(merged_flags, &effective_target.name);

            merged.value_declaration = source
                .value_declaration
                .clone()
                .or_else(|| effective_target.value_declaration.clone());

            merged.declarations = effective_target.declarations.clone();
            merged
                .declarations
                .extend(source.declarations.iter().cloned());

            merged.parent = effective_target.parent.clone();

            merged.members = SymbolTable {
                entries: effective_target.members.entries.clone(),
            };
            merged.exports = SymbolTable {
                entries: effective_target.exports.entries.clone(),
            };

            let result = Arc::new(merged);

            let mut result_mut = Symbol::new(result.flags, &result.name);
            result_mut.value_declaration = result.value_declaration.clone();
            result_mut.declarations = result.declarations.clone();
            result_mut.parent = result.parent.clone();
            result_mut.members = SymbolTable {
                entries: result.members.entries.clone(),
            };
            result_mut.exports = SymbolTable {
                entries: result.exports.entries.clone(),
            };

            let source_members: Vec<(String, Arc<Symbol>)> = source
                .members
                .iter()
                .map(|(k, v)| (k.clone(), Arc::clone(v)))
                .collect();
            for (name, source_sym) in source_members {
                if let Some(target_sym) = result_mut.members.entries.get_mut(&name) {
                    let merged = self.merge_symbol(target_sym, &source_sym, unidirectional);
                    *target_sym = merged;
                } else {
                    let merged = self.get_merged_symbol(&source_sym);
                    result_mut.members.insert(name, merged);
                }
            }

            let source_exports: Vec<(String, Arc<Symbol>)> = source
                .exports
                .iter()
                .map(|(k, v)| (k.clone(), Arc::clone(v)))
                .collect();
            for (name, source_sym) in source_exports {
                if let Some(target_sym) = result_mut.exports.entries.get_mut(&name) {
                    let merged = self.merge_symbol(target_sym, &source_sym, unidirectional);
                    *target_sym = merged;
                } else {
                    let merged = self.get_merged_symbol(&source_sym);
                    result_mut.exports.insert(name, merged);
                }
            }

            let final_result = Arc::new(result_mut);

            if !unidirectional {
                self.record_merged_symbol(&final_result, source);
            }

            final_result
        } else {

            self.report_merge_symbol_error(target, source);
            Arc::clone(target)
        }
    }

    fn report_merge_symbol_error(&mut self, target: &Arc<Symbol>, source: &Arc<Symbol>) {
        let is_either_enum =
            target.flags.contains(SymbolFlags::ENUM) || source.flags.contains(SymbolFlags::ENUM);
        let is_either_block_scoped = target
            .flags
            .intersects(SymbolFlags::BlockScopedVariable)
            || source.flags.intersects(SymbolFlags::BlockScopedVariable);
        let message = if is_either_enum {
            crate::diagnostics::messages_generated::
                ENUM_DECLARATIONS_CAN_ONLY_MERGE_WITH_NAMESPACE_OR_OTHER_ENUM_DECLARATIONS
        } else if is_either_block_scoped {
            crate::diagnostics::messages_generated::CANNOT_REDECLARE_BLOCK_SCOPED_VARIABLE_0
        } else {
            crate::diagnostics::messages_generated::DUPLICATE_IDENTIFIER_0
        };
        let name = source.name.clone();
        let mut locs: Vec<crate::core::text::TextRange> = Vec::new();
        for sym in [target, source] {
            for d in &sym.declarations {
                let name_node =
                    crate::ast::utilities::get_name_of_declaration(d).unwrap_or_else(|| Arc::clone(d));
                locs.push(name_node.loc);
            }
        }
        for loc in locs {

            if self
                .diagnostics
                .get_all()
                .iter()
                .any(|d| d.loc == loc && d.code == message.code)
            {
                continue;
            }
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                loc,
                message,
                vec![name.clone()],
            ));
        }
    }

    pub fn record_merged_symbol(&mut self, target: &Arc<Symbol>, source: &Arc<Symbol>) {
        self.merged_symbols.insert(source.id(), target.id());
    }

    pub fn clone_symbol(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
        let mut cloned = Symbol::new(symbol.flags | SymbolFlags::Transient, &symbol.name);
        cloned.declarations = symbol.declarations.clone();
        cloned.parent = symbol.parent.clone();
        cloned.value_declaration = symbol.value_declaration.clone();
        cloned.members = SymbolTable {
            entries: symbol.members.entries.clone(),
        };
        cloned.exports = SymbolTable {
            entries: symbol.exports.entries.clone(),
        };
        let result = Arc::new(cloned);
        Some(result)
    }

    pub fn resolve_symbol(&self, symbol: &Arc<Symbol>) -> Arc<Symbol> {
        if let Some(result) = self.follow_alias(symbol) {
            result
        } else {
            Arc::clone(symbol)
        }
    }

    pub(crate) fn get_symbol_at_location(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> {

        if let Some(sym) = self.program.symbol_map().symbol_of(node) {
            return Some(Arc::clone(sym));
        }

        if node.kind == crate::ast::SyntaxKind::Identifier {
            let mut current = node.parent.as_ref();
            while let Some(parent) = current {
                if let Some(sym) = self.program.symbol_map().symbol_of(parent) {
                    return Some(Arc::clone(sym));
                }
                current = parent.parent.as_ref();
            }
        }

        if node.kind == crate::ast::SyntaxKind::PropertyAccessExpression {
            if let crate::ast::NodeData::PropertyAccessExpression(data) = &node.data {
                if let Some(links) = self.type_node_links.get(&data.expression) {
                    if let Some(ref t) = links.resolved_type {
                        return self.get_property_of_type(t, data.name.text());
                    }
                }
            }
        }

        None
    }
}
