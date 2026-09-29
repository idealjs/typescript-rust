use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::scanner::CommentRange;
use super::m4q::r33k12_defs::{is_keyword_kind, is_punctuation_kind};
use super::m4r::*;

pub trait NodeR36k16Accessors {
    fn r16_flags(&self) -> NodeFlags;
    fn asterisk_token(&self) -> Option<&Node>;
    fn argument_expression(&self) -> &Node;
    fn tag(&self) -> &Node;
    fn template(&self) -> &Node;
    fn declaration_list(&self) -> &Node;
    fn declarations(&self) -> &NodeList;
    fn statement(&self) -> &Node;
    fn try_block(&self) -> &Node;
    fn catch_clause(&self) -> Option<&Node>;
    fn finally_block(&self) -> Option<&Node>;
    fn exclamation_token(&self) -> Option<&Node>;
    fn ty(&self) -> Option<&Node>;
}

impl NodeR36k16Accessors for Node {
    fn r16_flags(&self) -> NodeFlags {
        self.flags
    }

    fn asterisk_token(&self) -> Option<&Node> {
        match &self.data {
            NodeData::YieldExpression(d) => d.asterisk_token.as_deref(),
            _ => None,
        }
    }

    fn argument_expression(&self) -> &Node {
        match &self.data {
            NodeData::ElementAccessExpression(d) => &d.argument_expression,
            _ => panic!("unexpected ElementAccessExpression: {:?}", self.kind),
        }
    }

    fn tag(&self) -> &Node {
        match &self.data {
            NodeData::TaggedTemplateExpression(d) => &d.tag,
            _ => panic!("unexpected TaggedTemplateExpression: {:?}", self.kind),
        }
    }

    fn template(&self) -> &Node {
        match &self.data {
            NodeData::TaggedTemplateExpression(d) => &d.template,
            _ => panic!("unexpected TaggedTemplateExpression: {:?}", self.kind),
        }
    }

    fn declaration_list(&self) -> &Node {
        match &self.data {
            NodeData::VariableStatement(d) => &d.declaration_list,
            _ => panic!("unexpected VariableStatement: {:?}", self.kind),
        }
    }

    fn declarations(&self) -> &NodeList {
        match &self.data {
            NodeData::VariableDeclarationList(d) => &d.declarations,
            _ => panic!("unexpected VariableDeclarationList: {:?}", self.kind),
        }
    }

    fn statement(&self) -> &Node {
        match &self.data {
            NodeData::WhileStatement(d) => &d.statement,
            NodeData::DoStatement(d) => &d.statement,
            NodeData::WithStatement(d) => &d.statement,
            NodeData::ForStatement(d) => &d.statement,
            NodeData::ForInOrOfStatement(d) => &d.statement,
            NodeData::LabeledStatement(d) => &d.statement,
            _ => panic!("unexpected statement-bearing node: {:?}", self.kind),
        }
    }

    fn try_block(&self) -> &Node {
        match &self.data {
            NodeData::TryStatement(d) => &d.try_block,
            _ => panic!("unexpected TryStatement: {:?}", self.kind),
        }
    }

    fn catch_clause(&self) -> Option<&Node> {
        match &self.data {
            NodeData::TryStatement(d) => d.catch_clause.as_deref(),
            _ => None,
        }
    }

    fn finally_block(&self) -> Option<&Node> {
        match &self.data {
            NodeData::TryStatement(d) => d.finally_block.as_deref(),
            _ => None,
        }
    }

    fn exclamation_token(&self) -> Option<&Node> {
        match &self.data {
            NodeData::VariableDeclaration(d) => d.exclamation_token.as_deref(),
            _ => None,
        }
    }

    fn ty(&self) -> Option<&Node> {
        match &self.data {
            NodeData::VariableDeclaration(d) => d.type_node.as_deref(),
            NodeData::TypeAliasDeclaration(d) => Some(d.type_node.as_ref()),
            _ => None,
        }
    }
}

pub trait EmitContextR36k16 {
    fn factory(&self) -> tsox_frontend::format::mig::m4o::NodeFactory;
    fn set_original(&self, node: &Arc<Node>, original: &Node);
    fn get_synthetic_leading_comments(
        &self,
        node: &Node,
    ) -> Vec<crate::printer::mig::m4m_2::SynthesizedComment>;
    fn get_external_helpers_module_name(&self, source_file: &Node) -> Option<Arc<Node>>;
}

impl EmitContextR36k16 for tsox_frontend::format::mig::m4o_2::EmitContext {
    fn factory(&self) -> tsox_frontend::format::mig::m4o::NodeFactory {
        tsox_frontend::format::mig::m4o::new_node_factory(
            tsox_frontend::format::mig::m4o::EmitContext::default(),
        )
    }

    fn set_original(&self, _node: &Arc<Node>, _original: &Node) {
        // Go SetOriginal 写入 emitNode.original;Rust 侧 per-node 侧表尚在 m4o 迁移中,交接记待接。
    }

    fn get_synthetic_leading_comments(
        &self,
        _node: &Node,
    ) -> Vec<crate::printer::mig::m4m_2::SynthesizedComment> {
        Vec::new()
    }

    fn get_external_helpers_module_name(&self, _source_file: &Node) -> Option<Arc<Node>> {
        None
    }
}

impl Printer {
    pub fn write_space(&mut self) {
        self.writer.write_space(" ");
    }

    pub fn write_line(&mut self) {
        self.writer.write_line();
    }

    pub fn write_line_repeat(&mut self, count: i32) {
        for _ in 0..count {
            self.write_line();
        }
    }

    pub fn write_punctuation(&mut self, text: &str) {
        self.writer.write_punctuation(text);
    }

    pub fn write_keyword(&mut self, text: &str) {
        self.writer.write_keyword(text);
    }

    pub fn write_trailing_semicolon(&mut self) {
        self.writer.write_trailing_semicolon(";");
    }

    pub fn write_token_text(&mut self, token: SyntaxKind, write_kind: WriteKind, pos: usize) -> usize {
        let text = tsox_frontend::scanner::token_to_string(token);
        self.write_as(text, write_kind);
        pos + text.len()
    }

    pub fn emit_token(
        &mut self,
        token: SyntaxKind,
        pos: usize,
        write_kind: WriteKind,
        context_node: &Node,
    ) -> usize {
        self.emit_token_ex(token, pos, write_kind, context_node, TEF_NONE)
    }

    pub fn emit_token_ex(
        &mut self,
        token: SyntaxKind,
        pos: usize,
        write_kind: WriteKind,
        context_node: &Node,
        flags: TokenEmitFlags,
    ) -> usize {
        let state = self.enter_token_node(context_node, flags);
        let pos = self.write_token_text(token, write_kind, pos);
        self.exit_token_node(context_node, state);
        pos
    }

    pub fn emit_keyword_node(&mut self, node: Option<&Node>) {
        self.emit_keyword_node_ex(node, TEF_NONE);
    }

    pub fn emit_keyword_node_ex(&mut self, node: Option<&Node>, flags: TokenEmitFlags) {
        let Some(node) = node else {
            return;
        };
        let state = self.enter_token_node(node, flags);
        self.write_token_text(node.kind, WriteKind::Keyword, node.pos());
        self.exit_token_node(node, state);
    }

    pub fn emit_punctuation_node(&mut self, node: Option<&Node>) {
        self.emit_punctuation_node_ex(node, TEF_NONE);
    }

    pub fn emit_punctuation_node_ex(&mut self, node: Option<&Node>, flags: TokenEmitFlags) {
        let Some(node) = node else {
            return;
        };
        let state = self.enter_token_node(node, flags);
        self.write_token_text(node.kind, WriteKind::Punctuation, node.pos());
        self.exit_token_node(node, state);
    }

    pub fn emit_token_node(&mut self, node: Option<&Node>) {
        self.emit_token_node_ex(node, TEF_NONE);
    }

    pub fn emit_token_node_ex(&mut self, node: Option<&Node>, flags: TokenEmitFlags) {
        let Some(node) = node else {
            return;
        };
        if is_keyword_kind(node.kind) {
            self.emit_keyword_node_ex(Some(node), flags);
        } else if is_punctuation_kind(node.kind) {
            self.emit_punctuation_node_ex(Some(node), flags);
        } else {
            panic!("unexpected TokenNode: {:?}", node.kind);
        }
    }

    pub fn comment_will_emit_new_line(&self, comment: CommentRange) -> bool {
        comment.kind == tsox_frontend::scanner::CommentRangeKind::SingleLine
            || comment.has_trailing_new_line
    }

    pub fn write_line_or_space(&mut self, parent_node: &Node, prev_child_node: &Node, next_child_node: &Node) {
        if self.should_emit_on_single_line(parent_node) {
            self.write_space();
        } else if self.options.preserve_source_newlines {
            let lines = self.get_lines_between_nodes(parent_node, prev_child_node, next_child_node);
            if lines > 0 {
                self.write_line_repeat(lines);
            } else {
                self.write_space();
            }
        } else {
            self.write_line();
        }
    }
}
