#![allow(unused_imports)]

use super::*;

impl Parser {
    pub(crate) fn make_modifier_list(
        &self,
        modifiers: Vec<(SyntaxKind, usize, usize)>,
    ) -> Arc<ModifierList> {
        let mut flags = ModifierFlags::empty();
        let nodes = modifiers
            .into_iter()
            .map(|(kind, pos, end)| {
                flags |= Self::modifier_flag(kind);
                Arc::new(Node::with_loc(
                    kind,
                    NodeData::Token,
                    TextRange::new(pos, end),
                ))
            })
            .collect();
        Arc::new(ModifierList::new(nodes, flags))
    }

    pub(crate) fn make_modifier_list_with_decorators(
        &self,
        modifiers: Vec<(SyntaxKind, usize, usize)>,
        decorators: Vec<Arc<Node>>,
    ) -> Arc<ModifierList> {
        let mut flags = ModifierFlags::empty();
        let mut nodes: Vec<Arc<Node>> = Vec::with_capacity(modifiers.len() + decorators.len());
        for (kind, pos, end) in modifiers {
            flags |= Self::modifier_flag(kind);
            nodes.push(Arc::new(Node::with_loc(
                kind,
                NodeData::Token,
                TextRange::new(pos, end),
            )));
        }
        if !decorators.is_empty() {
            flags |= ModifierFlags::Decorator;
            nodes.extend(decorators);
        }
        Arc::new(ModifierList::new(nodes, flags))
    }

    pub(crate) fn parse_decorator(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::AtToken);
        let expression = self.parse_left_hand_side_expression();
        let end = expression.end();
        Arc::new(Node::with_loc(
            SyntaxKind::Decorator,
            NodeData::Decorator(DecoratorData { expression }),
            TextRange::new(pos, end),
        ))
    }
}
