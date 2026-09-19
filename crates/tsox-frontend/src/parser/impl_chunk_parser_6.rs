#![allow(unused_imports)]

use crate::parser::impl_chunk::*;

impl Parser {
    /// Go 声明节点 span 起于首修饰符/装饰符（statement 级 pos 在修饰符前捕获），
    /// Rust 各声明解析入口在关键字处取 pos，此处向前扩展覆盖修饰符。
    pub(crate) fn declaration_start(
        modifiers: &Option<Arc<ModifierList>>,
        keyword_pos: usize,
    ) -> usize {
        modifiers
            .as_ref()
            .and_then(|m| m.nodes.first().map(|n| n.pos()))
            .map_or(keyword_pos, |p| p.min(keyword_pos))
    }

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

    pub(crate) fn make_optional_modifier_list(
        &self,
        modifiers: &[(SyntaxKind, usize, usize)],
    ) -> Option<Arc<ModifierList>> {
        if modifiers.is_empty() {
            None
        } else {
            Some(self.make_modifier_list(modifiers.to_vec()))
        }
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
        // Go doInContext(NodeFlagsDecoratorContext)：装饰器表达式内不算元素访问
        let save_decorator_context = self.decorator_context;
        self.decorator_context = true;
        let expression = self.parse_left_hand_side_expression();
        self.decorator_context = save_decorator_context;
        let end = expression.end();
        Arc::new(Node::with_loc(
            SyntaxKind::Decorator,
            NodeData::Decorator(DecoratorData { expression }),
            TextRange::new(pos, end),
        ))
    }
}
