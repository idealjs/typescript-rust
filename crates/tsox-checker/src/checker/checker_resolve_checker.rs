#![allow(unused_imports)]

use crate::checker::checker_resolve::*;

pub(crate) const ANCESTRY_CONTAINERS: &[SyntaxKind] = &[
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

pub(crate) fn lexical_scope_chain_ids(node: &Arc<Node>) -> Vec<u64> {
    let mut chain = Vec::new();
    let mut ancestor = node.parent();
    while let Some(a) = ancestor {
        if ANCESTRY_CONTAINERS.contains(&a.kind) {
            chain.push(a.id());
        }
        ancestor = a.parent();
    }
    chain
}

impl Checker {
    pub fn resolve_identifier(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        self.resolve_identifier_with_meaning(node, SymbolFlags::all())
    }

    pub fn resolve_identifier_with_meaning(
        &self,
        node: &Arc<Node>,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        self.resolve_identifier_scope_symbol(node, meaning)
            .and_then(|s| self.follow_alias(&s))
    }

    pub fn resolve_identifier_use(
        &self,
        node: &Arc<Node>,
        record_meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        let scope = self.resolve_identifier_scope_symbol(node, SymbolFlags::all());
        let result = scope.as_ref().and_then(|s| self.follow_alias(s));
        if let Some(sym) = &scope
            && access_kind(node) != AccessKind::Write
            && !self.self_reference_location_exempts(node, sym)
        {
            self.record_symbol_reference(sym, record_meaning);
        }
        result
    }

    fn self_reference_location_exempts(&self, node: &Arc<Node>, resolved: &Arc<Symbol>) -> bool {
        let name = match &node.data {
            tsox_frontend::ast::NodeData::Identifier(data) => data.text.as_str(),
            _ => return false,
        };
        let symbol_map = self.program.symbol_map();
        let mut last_location: Option<Arc<Node>> = None;
        let mut last_self_reference: Option<Arc<Node>> = None;
        let mut current = node.parent();
        while let Some(location) = current {
            if let Some(locals) = symbol_map.locals.get(&location.id())
                && locals.get(name).is_some()
            {
                break;
            }
            let is_self_reference = match location.kind {
                SyntaxKind::Parameter => last_location
                    .as_ref()
                    .zip(location.name())
                    .is_some_and(|(last, name)| Arc::ptr_eq(last, &name)),
                SyntaxKind::FunctionDeclaration
                | SyntaxKind::ClassDeclaration
                | SyntaxKind::InterfaceDeclaration
                | SyntaxKind::EnumDeclaration
                | SyntaxKind::TypeAliasDeclaration
                | SyntaxKind::ModuleDeclaration => true,
                _ => false,
            };
            if is_self_reference && last_self_reference.is_none() {
                last_self_reference = Some(Arc::clone(&location));
            }
            last_location = Some(Arc::clone(&location));
            current = location.parent();
        }
        last_self_reference.is_some_and(|location| {
            symbol_map
                .symbol_of(&location)
                .is_some_and(|sym| sym.id() == resolved.id())
        })
    }

    pub(crate) fn record_symbol_reference(&self, symbol: &Arc<Symbol>, bits: SymbolFlags) {
        self.symbol_reference_kinds
            .entry(symbol.id())
            .and_modify(|f| *f |= bits)
            .or_insert(bits);
    }

    pub(crate) fn alias_chain_hits_meaning(&self, sym: &Arc<Symbol>, meaning: SymbolFlags) -> bool {
        if !sym.flags.intersects(SymbolFlags::Alias) {
            return false;
        }
        match self.follow_alias(sym) {
            Some(target) if !Arc::ptr_eq(&target, sym) => target.flags.intersects(meaning),
            _ => true,
        }
    }

    pub fn follow_alias(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
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
}

impl Checker {
    /// Go getVisibleSymbolInAugmentationScope：declare module "foo"（augmentation）
    /// 块内的裸名查找延伸到被增强模块的 exports
    pub(crate) fn augmentation_target_member(
        &self,
        container_sym: &Arc<Symbol>,
        name: &str,
    ) -> Option<Arc<Symbol>> {
        let decl = container_sym
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ModuleDeclaration)?;
        let tsox_frontend::ast::NodeData::ModuleDeclaration(md) = &decl.data else {
            return None;
        };
        if md.name.kind != SyntaxKind::StringLiteral {
            return None;
        }
        let spec = md.name.text().trim_matches(['"', '\'']).to_string();
        let resolved = self.resolve_module_file_symbol(&spec)?;
        let target = if Arc::ptr_eq(&resolved, container_sym) {
            return None;
        } else {
            resolved
        };
        target
            .exports
            .get(name)
            .cloned()
            .or_else(|| target.members.get(name).cloned())
    }
}

pub(crate) fn node_parent_is_require_call(node: &Arc<Node>) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    if let tsox_frontend::ast::NodeData::CallExpression(call) = &parent.data
        && call.expression.kind == SyntaxKind::Identifier
        && call.expression.text() == "require"
        && call.arguments.len() == 1
    {} else {
        return false;
    }
    let mut ancestor = Some(Arc::clone(node));
    while let Some(a) = ancestor {
        if a.kind == SyntaxKind::SourceFile {
            return a
                .flags
                .contains(tsox_frontend::ast::NodeFlags::JavaScriptFile);
        }
        ancestor = a.parent();
    }
    false
}
