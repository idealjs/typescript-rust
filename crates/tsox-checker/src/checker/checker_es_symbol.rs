#![allow(unused_imports)]

use crate::checker::checker_impl_chunk::*;

impl Checker {
    pub fn get_es_symbol_like_type_for_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let Some(symbol) = self.valid_es_symbol_declaration_symbol(node) else {
            return self.es_symbol_type();
        };
        let key = symbol.id();
        if let Some(t) = self.unique_es_symbol_types.get(&key) {
            return Arc::clone(t);
        }
        let t = Arc::new(Type {
            flags: TypeFlags::UniqueESSymbol,
            object_flags: ObjectFlags::None,
            id: crate::checker::types::next_type_id(),
            symbol: Some(Arc::clone(&symbol)),
            alias: None,
            data: TypeData::UniqueESSymbol(UniqueESSymbolTypeData {
                name: format!("__@{}", symbol.name),
            }),
        });
        self.unique_es_symbol_types.insert(key, Arc::clone(&t));
        t
    }

    fn valid_es_symbol_declaration_symbol(&mut self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        match &node.data {
            NodeData::VariableDeclaration(data) => {
                if !matches!(data.name.data, NodeData::Identifier(_)) {
                    return None;
                }
                let list = node.parent()?;
                if list.kind != SyntaxKind::VariableDeclarationList {
                    return None;
                }
                let stmt = list.parent()?;
                if stmt.kind != SyntaxKind::VariableStatement {
                    return None;
                }
                if !self
                    .get_combined_node_flags(node)
                    .intersects(NodeFlags::Constant)
                {
                    return None;
                }
            }
            NodeData::PropertyDeclaration(data) => {
                let flags = data
                    .modifiers
                    .as_ref()
                    .map_or(ModifierFlags::empty(), |m| m.modifier_flags);
                if !flags.contains(ModifierFlags::Readonly | ModifierFlags::Static) {
                    return None;
                }
            }
            NodeData::PropertySignatureDeclaration(data) => {
                let flags = data
                    .modifiers
                    .as_ref()
                    .map_or(ModifierFlags::empty(), |m| m.modifier_flags);
                if !flags.contains(ModifierFlags::Readonly) {
                    return None;
                }
            }
            _ => return None,
        }
        self.program.symbol_map().symbol_of(node).map(Arc::clone)
    }

    pub fn is_symbol_or_symbol_for_call(&self, node: &Arc<Node>) -> bool {
        let NodeData::CallExpression(data) = &node.data else {
            return false;
        };
        let mut left = &data.expression;
        if let NodeData::PropertyAccessExpression(pa) = &left.data
            && matches!(&pa.name.data, NodeData::Identifier(id) if id.text == "for")
        {
            left = &pa.expression;
        }
        let NodeData::Identifier(id) = &left.data else {
            return false;
        };
        if id.text != "Symbol" {
            return false;
        }
        let Some(global_symbol) = self.globals.get("Symbol") else {
            return false;
        };
        self.resolve_identifier(left)
            .is_some_and(|sym| Arc::ptr_eq(&sym, global_symbol))
    }

    pub fn widen_unique_symbol_for_declaration(
        &mut self,
        declaration: &Arc<Node>,
        t: &Arc<Type>,
    ) -> Arc<Type> {
        if !t.flags.contains(TypeFlags::UniqueESSymbol) {
            return Arc::clone(t);
        }
        if declaration.kind != SyntaxKind::BindingElement
            && declaration.kind != SyntaxKind::VariableDeclaration
            && declaration.kind != SyntaxKind::PropertyDeclaration
        {
            return Arc::clone(t);
        }
        let decl_symbol = self.program.symbol_map().symbol_of(declaration);
        if t.symbol.as_ref().zip(decl_symbol.as_ref()).is_some_and(
            |(ts, ds)| {
                Arc::ptr_eq(ts, ds)
            },
        ) {
            return Arc::clone(t);
        }
        self.es_symbol_type()
    }
}

pub(crate) fn walk_up_parenthesized_types(node: &Arc<Node>) -> Arc<Node> {
    let mut current = Arc::clone(node);
    while current.kind == SyntaxKind::ParenthesizedType {
        let Some(parent) = current.parent() else {
            break;
        };
        current = parent;
    }
    current
}

pub(crate) fn walk_up_parenthesized_expressions(node: &Arc<Node>) -> Arc<Node> {
    let mut current = Arc::clone(node);
    while current.kind == SyntaxKind::ParenthesizedExpression {
        let Some(parent) = current.parent() else {
            break;
        };
        current = parent;
    }
    current
}
