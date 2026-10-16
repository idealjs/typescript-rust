#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;

use super::m4r::*;
use super::m4r_3::{EmitContextR36k16, NodeR36k16Accessors};
use tsox_core::tspath::file_extension_is;
use tsox_frontend::ast::node_data_generated::is_binding_pattern;
use tsox_frontend::ast::utilities::is_member_name;
use tsox_frontend::scanner::mig::w1::compute_line_of_position;
use tsox_frontend::ast::mig::m3c;
use tsox_frontend::ast::mig::m3g_2::is_parse_tree_node;
use tsox_frontend::ast::node_source_file::SourceFile;
use tsox_frontend::ast::{Node, NodeList, SyntaxKind};
use tsox_frontend::scanner::CommentRange;

pub trait SourceMapGeneratorK06 {
    fn add_source(&mut self, source_file_path: &str, next_index: SourceIndex) -> SourceIndex;
    fn set_source_content(&mut self, source_index: SourceIndex, contents: &str);
}

impl SourceMapGeneratorK06 for SourceMapGenerator {
    fn add_source(&mut self, _source_file_path: &str, next_index: SourceIndex) -> SourceIndex { ::tsox_core::fntrace::enter("add_source"); 
        next_index
    }

    fn set_source_content(&mut self, _source_index: SourceIndex, _contents: &str) { ::tsox_core::fntrace::enter("set_source_content"); }
}

impl Printer {
    pub fn should_allow_trailing_comma(&self, node: &Node, list: &NodeList) -> bool { ::tsox_core::fntrace::enter("should_allow_trailing_comma"); 
        if self.current_source_file.is_none()
            || self.current_source_file.as_deref().unwrap().script_kind == SCRIPT_KIND_JSON
        {
            return false;
        }
        should_allow_trailing_comma_worker(node, list)
    }

    pub fn get_unique_helper_name(&mut self, name: &str) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_unique_helper_name"); 
        let helper_name = self.unique_helper_names.get(name).cloned().flatten();
        if helper_name.is_none() {
            // Go: emitContext.Factory.NewUniqueNameEx(name, FileLevel|Optimistic)。唯一名分配
            // 在 crate::printer 的借用版 NodeFactory<'a>::new_unique_name_ex 上（需 printer::EmitContext
            // 引用，m4r::Printer 未持有），接线前先以基础名建 Identifier（交接）。
            let helper_name = self.emit_context.factory().new_identifier(name);
            self.generate_name(&helper_name);
            self.unique_helper_names
                .insert(name.to_string(), Some(helper_name.clone()));
            return Some(helper_name);
        }
        let helper_name = helper_name.unwrap();
        Some(self.emit_context.factory().new_identifier(helper_name.text()))
    }

    pub fn emit_trailing_synthetic_comments_of_node(&mut self, node: &Node, emit_flags: EmitFlags) { ::tsox_core::fntrace::enter("emit_trailing_synthetic_comments_of_node"); 
        if emit_flags & EF_NO_TRAILING_COMMENTS != 0 {
            return;
        }
        let synth = self.emit_context.k06_get_synthetic_trailing_comments(node);
        for c in synth {
            self.emit_trailing_synthesized_comment(&c);
        }
    }

    pub fn emit_trailing_synthesized_comment(&mut self, comment: &SynthesizedComment) { ::tsox_core::fntrace::enter("emit_trailing_synthesized_comment"); 
        if !self.writer.is_at_start_of_line() {
            self.writer.write_space(" ");
        }
        self.k06_write_synthesized_comment(comment);
        if comment.has_trailing_new_line {
            self.writer.write_line();
        }
    }

    pub fn should_emit_comment_if_triple_slash(&self, comment: CommentRange, triple_slash: Tristate) -> bool { ::tsox_core::fntrace::enter("should_emit_comment_if_triple_slash"); 
        match triple_slash {
            Tristate::True => self.is_triple_slash_comment(comment),
            Tristate::False => !self.is_triple_slash_comment(comment),
            Tristate::Unknown => true,
        }
    }

    pub fn should_emit_new_line_before_leading_comment_of_position(&self, pos: i32, comment_pos: i32) -> bool { ::tsox_core::fntrace::enter("should_emit_new_line_before_leading_comment_of_position"); 
        let Some(source_file) = self.current_source_file.as_deref() else {
            return false;
        };
        pos != comment_pos
            && compute_line_of_position(&source_file.ecma_line_map(), pos)
                != compute_line_of_position(&source_file.ecma_line_map(), comment_pos)
    }

    pub fn is_triple_slash_comment(&self, comment: CommentRange) -> bool { ::tsox_core::fntrace::enter("is_triple_slash_comment"); 
        self.current_source_file.is_some()
            && is_recognized_triple_slash_comment(
                self.current_source_file.as_deref().unwrap().text(),
                &comment,
            )
    }

    pub fn set_source_map_source(&mut self, source: &Arc<SourceFile>) { ::tsox_core::fntrace::enter("set_source_map_source"); 
        if self.source_maps_disabled {
            return;
        }

        self.source_map_source = Some(source.clone());
        self.source_map_line_char_cache = Some(tsox_frontend::format::mig::m4t_3::LineCharacterCache {
            line_map: source.ecma_line_map(),
            text: source.text().to_string(),
            cached_line: 0,
            cached_pos: 0,
            cached_char: 0,
            has_cached: false,
        });
        if self
            .most_recent_source_map_source
            .as_ref()
            .map_or(false, |s| Arc::ptr_eq(s, source))
        {
            self.source_map_source_index = self.most_recent_source_map_source_index;
            return;
        }

        self.source_map_source_is_json = file_extension_is(&source.file_name, EXTENSION_JSON);
        if self.source_map_source_is_json {
            return;
        }

        self.source_map_source_index = self
            .source_map_generator
            .as_mut()
            .unwrap()
            .add_source(&source.file_name, self.source_map_source_index + 1);
        if self.options.inline_sources {
            self.source_map_generator
                .as_mut()
                .unwrap()
                .set_source_content(self.source_map_source_index, source.text());
        }

        self.most_recent_source_map_source = Some(source.clone());
        self.most_recent_source_map_source_index = self.source_map_source_index;
    }

    pub fn should_reuse_temp_variable_scope(&self, node: Option<&Node>) -> bool { ::tsox_core::fntrace::enter("should_reuse_temp_variable_scope"); 
        node.is_some()
            && self.emit_context.k06_emit_flags(node.unwrap()) & (EF_REUSE_TEMP_VARIABLE_SCOPE as u32) != 0
    }

    pub fn push_name_generation_scope(&mut self, node: Option<&Node>) { ::tsox_core::fntrace::enter("push_name_generation_scope"); 
        let reuse = self.should_reuse_temp_variable_scope(node);
        self.name_generator.push_scope(reuse);
    }

    pub fn pop_name_generation_scope(&mut self, node: Option<&Node>) { ::tsox_core::fntrace::enter("pop_name_generation_scope"); 
        let reuse = self.should_reuse_temp_variable_scope(node);
        self.name_generator.pop_scope(reuse);
    }

    pub fn generate_all_names(&mut self, nodes: Option<&NodeList>) { ::tsox_core::fntrace::enter("generate_all_names"); 
        let Some(nodes) = nodes else {
            return;
        };
        for node in &nodes.nodes {
            self.generate_names(Some(node));
        }
    }

    pub fn generate_names(&mut self, node: Option<&Node>) { ::tsox_core::fntrace::enter("generate_names"); 
        let Some(node) = node else {
            return;
        };

        match node.kind {
            SyntaxKind::Block | SyntaxKind::CaseClause | SyntaxKind::DefaultClause => {
                self.generate_all_names(m3c::statement_list(node).map(|l| &**l))
            }
            SyntaxKind::LabeledStatement
            | SyntaxKind::WithStatement
            | SyntaxKind::DoStatement
            | SyntaxKind::WhileStatement => self.generate_names(Some(node.statement())),
            SyntaxKind::IfStatement => {
                self.generate_names(Some(node.k06_then_statement()));
                self.generate_names(node.k06_else_statement());
            }
            SyntaxKind::ForStatement | SyntaxKind::ForOfStatement | SyntaxKind::ForInStatement => {
                self.generate_names(node.initializer().map(|v| &**v));
                self.generate_names(Some(node.statement()));
            }
            SyntaxKind::SwitchStatement => self.generate_names(Some(node.k06_case_block())),
            SyntaxKind::CaseBlock => self.generate_all_names(Some(node.k06_clauses())),
            SyntaxKind::TryStatement => {
                self.generate_names(Some(node.try_block()));
                self.generate_names(node.catch_clause());
                self.generate_names(node.finally_block());
            }
            SyntaxKind::CatchClause => {
                self.generate_names(node.k06_variable_declaration());
                self.generate_names(Some(node.k06_block()));
            }
            SyntaxKind::VariableStatement => self.generate_names(Some(node.declaration_list())),
            SyntaxKind::VariableDeclarationList => self.generate_all_names(Some(node.declarations())),
            SyntaxKind::VariableDeclaration
            | SyntaxKind::Parameter
            | SyntaxKind::BindingElement
            | SyntaxKind::ClassDeclaration => self.generate_name_if_needed(node.name().map(|v| &**v)),
            SyntaxKind::FunctionDeclaration => {
                self.generate_name_if_needed(node.name().map(|v| &**v));
                if self.should_reuse_temp_variable_scope(Some(node)) {
                    self.generate_all_names(node.parameters().map(|v| &**v));
                    self.generate_names(node.body().map(|v| &**v));
                }
            }
            SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern => {
                self.generate_all_names(Some(node.k06_element_list()))
            }
            SyntaxKind::ImportDeclaration | SyntaxKind::JSImportDeclaration => {
                self.generate_names(node.k06_import_clause())
            }
            SyntaxKind::ImportClause => {
                self.generate_name_if_needed(node.name().map(|v| &**v));
                self.generate_names(node.k06_named_bindings());
            }
            SyntaxKind::NamespaceImport | SyntaxKind::NamespaceExport => {
                self.generate_name_if_needed(node.name().map(|v| &**v))
            }
            SyntaxKind::NamedImports => self.generate_all_names(Some(node.k06_element_list())),
            SyntaxKind::ImportSpecifier => {
                if node.property_name().is_some() {
                    self.generate_name_if_needed(node.property_name().map(|v| &**v));
                } else {
                    self.generate_name_if_needed(node.name().map(|v| &**v));
                }
            }
            _ => {}
        }
    }

    pub fn generate_all_member_names(&mut self, nodes: Option<&NodeList>) { ::tsox_core::fntrace::enter("generate_all_member_names"); 
        let Some(nodes) = nodes else {
            return;
        };
        for node in &nodes.nodes {
            self.generate_member_names(Some(node));
        }
    }

    pub fn generate_member_names(&mut self, node: Option<&Node>) { ::tsox_core::fntrace::enter("generate_member_names"); 
        let Some(node) = node else {
            return;
        };
        match node.kind {
            SyntaxKind::PropertyAssignment
            | SyntaxKind::ShorthandPropertyAssignment
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor => self.generate_name_if_needed(node.name().map(|v| &**v)),
            _ => {}
        }
    }

    pub fn generate_name_if_needed(&mut self, name: Option<&Node>) { ::tsox_core::fntrace::enter("generate_name_if_needed"); 
        if let Some(name) = name {
            if is_member_name(name) {
                self.generate_name(name);
            } else if is_binding_pattern(name) {
                self.generate_names(Some(name));
            }
        }
    }

    pub fn generate_name(&mut self, name: &Node) { ::tsox_core::fntrace::enter("generate_name"); 
        let _ = name.text();
    }

    pub fn is_file_level_unique_name_in_current_file(&self, name: &str, _scoped: bool) -> bool { ::tsox_core::fntrace::enter("is_file_level_unique_name_in_current_file"); 
        if let Some(source_file) = self.current_source_file.as_deref() {
            self.emit_context
                .is_file_level_unique_name(source_file, name, self.has_global_name_callback())
        } else {
            true
        }
    }

    fn has_global_name_callback(&self) -> Option<fn(&str) -> bool> { ::tsox_core::fntrace::enter("has_global_name_callback"); 
        None
    }

    pub fn emit_comments_before_node(&mut self, node: &Node) -> Option<CommentState> { ::tsox_core::fntrace::enter("emit_comments_before_node"); 
        if !self.should_emit_comments(node) {
            return None;
        }
        // Go: emitLeadingCommentsOfNode + emitLeadingSyntheticCommentsOfNode + NO_NESTED_COMMENTS 置位 +
        // commentStateArena 携带状态。m4r::Printer 注释发射簇与 m4o_2::CommentState pub 构造器均未落地（交接）。
        None
    }

    pub fn emit_comments_after_node(&mut self, _node: &Node, previous_state: Option<CommentState>) { ::tsox_core::fntrace::enter("emit_comments_after_node"); 
        if previous_state.is_none() {
            return;
        }
        // Go: NO_NESTED_COMMENTS 复位 + emitTrailingSyntheticCommentsOfNode + emitTrailingCommentsOfNode +
        // GetTypeNode 尾随注释。状态字段 pub(crate) 不可读，随构造器一并交接。
    }

    pub fn emit_source_maps_before_node(&mut self, node: &Node) -> Option<SourceMapState> { ::tsox_core::fntrace::enter("emit_source_maps_before_node"); 
        if !self.should_emit_source_maps(node) {
            return None;
        }
        // Go: emitSourcePos(skipTrivia(loc.Pos())) + NO_NESTED_SOURCE_MAPS 置位 + sourceMapState 携带。
        // emit_pos 依赖 m4o_2::Generator 实体（unit 桩无 sources 存储），state 无 pub 构造器（交接）。
        None
    }

    pub fn emit_source_maps_after_node(&mut self, _node: &Node, previous_state: Option<SourceMapState>) { ::tsox_core::fntrace::enter("emit_source_maps_after_node"); 
        if previous_state.is_none() {
            return;
        }
        // Go: NO_NESTED_SOURCE_MAPS 复位 + 尾随 emitSourcePos。随 Generator 实体与 state 构造器一并交接。
    }

    pub fn emit_comments_before_token(
        &mut self,
        _token: SyntaxKind,
        pos: i32,
        context_node: &Node,
        flags: TokenEmitFlags,
    ) -> (Option<CommentState>, i32) { ::tsox_core::fntrace::enter("emit_comments_before_token"); 
        if flags & TEF_NO_COMMENTS != 0 || self.comments_disabled {
            let mut pos = pos;
            if self.current_source_file.is_some() && pos >= 0 {
                pos = tsox_frontend::scanner::skip_trivia(
                    self.current_source_file.as_deref().unwrap().text(),
                    pos as usize,
                ) as i32;
            }
            return (None, pos);
        }
        let start_pos = pos;
        let mut pos = pos;
        if let Some(source_file) = self.current_source_file.as_deref() {
            pos = tsox_frontend::scanner::skip_trivia(source_file.text(), start_pos as usize) as i32;
        }
        // Go: node := p.emitContext.ParseNode(contextNode); isSimilarNode := node != nil && node.Kind == contextNode.Kind。
        // ParseNode = MostOriginal + IsParseTreeNode；m4o_2::EmitContext 无 originals 表（set_original 为存根），
        // MostOriginal 恒等返回原节点，判定等价于对 contextNode 本身做 IsParseTreeNode（交接：originals 接线后回归 parse_node）。
        if !is_parse_tree_node(context_node) {
            return (None, pos);
        }
        if context_node.pos() != start_pos as usize {
            // Go: increaseIndentIf(needsIndent) + emitLeadingComments(startPos, elided=false) + decreaseIndentIf。
            // emitLeadingComments 属注释发射簇，未移植（交接）。
        }
        (None, pos)
    }

    pub fn emit_comments_after_token(
        &mut self,
        _token: SyntaxKind,
        _pos: i32,
        _context_node: &Node,
        previous_state: Option<CommentState>,
    ) { ::tsox_core::fntrace::enter("emit_comments_after_token"); 
        if previous_state.is_none() {
            return;
        }
        // Go: contextNode.End() != pos 时 emitTrailingComments(pos, 分隔符按 JsxExpression 分流)。
        // emitTrailingComments 属注释发射簇，未移植（交接）。
    }

    pub fn emit_source_maps_before_token(
        &mut self,
        token: SyntaxKind,
        pos: i32,
        context_node: &Node,
        flags: TokenEmitFlags,
    ) -> Option<SourceMapState> { ::tsox_core::fntrace::enter("emit_source_maps_before_token"); 
        if !self.should_emit_token_source_maps(token, pos, context_node, flags) {
            return None;
        }
        // Go: TokenSourceMapRange 命中则取 loc.Pos()，skipTrivia 后 emitSourcePos，state 携带 {emitFlags, loc, hasLoc}。
        // emit_pos 与 state 携带依赖 Generator 实体与构造器（交接）。
        None
    }

    pub fn emit_source_maps_after_token(
        &mut self,
        _token: SyntaxKind,
        _pos: i32,
        _context_node: &Node,
        previous_state: Option<SourceMapState>,
    ) { ::tsox_core::fntrace::enter("emit_source_maps_after_token"); 
        if previous_state.is_none() {
            return;
        }
        // Go: NO_TOKEN_TRAILING_SOURCE_MAPS 检查 + emitSourcePos(loc.End())。随实体与构造器交接。
    }

    pub fn enter_node(&mut self, node: &Node) -> PrinterState39k06 { ::tsox_core::fntrace::enter("enter_node"); 
        let mut state = PrinterState39k06::default();

        if let Some(on_before_emit_node) = &mut self.on_before_emit_node {
            on_before_emit_node(Some(node));
        }

        state.comment_state = self.emit_comments_before_node(node);
        state.source_map_state = self.emit_source_maps_before_node(node);
        state
    }

    pub fn exit_node(&mut self, node: &Node, previous_state: PrinterState39k06) { ::tsox_core::fntrace::enter("exit_node"); 
        self.emit_source_maps_after_node(node, previous_state.source_map_state);
        self.emit_comments_after_node(node, previous_state.comment_state);

        if let Some(on_after_emit_node) = &mut self.on_after_emit_node {
            on_after_emit_node(Some(node));
        }
    }

    pub fn enter_token_node(&mut self, node: &Node, flags: TokenEmitFlags) -> PrinterState39k06 { ::tsox_core::fntrace::enter("enter_token_node"); 
        let mut state = PrinterState39k06::default();

        if let Some(on_before_emit_token) = &mut self.on_before_emit_token {
            on_before_emit_token(Some(node));
        }

        if flags & TEF_NO_COMMENTS == 0 {
            state.comment_state = self.emit_comments_before_node(node);
        }
        if flags & TEF_NO_SOURCE_MAPS == 0 {
            state.source_map_state = self.emit_source_maps_before_node(node);
        }
        state
    }

    pub fn exit_token_node(&mut self, node: &Node, previous_state: PrinterState39k06) { ::tsox_core::fntrace::enter("exit_token_node"); 
        self.emit_source_maps_after_node(node, previous_state.source_map_state);
        self.emit_comments_after_node(node, previous_state.comment_state);

        if let Some(on_after_emit_token) = &mut self.on_after_emit_token {
            on_after_emit_token(Some(node));
        }
    }

    pub fn enter_token(
        &mut self,
        token: SyntaxKind,
        pos: i32,
        context_node: &Node,
        flags: TokenEmitFlags,
    ) -> (PrinterState39k06, i32) { ::tsox_core::fntrace::enter("enter_token"); 
        let mut state = PrinterState39k06::default();
        let (comment_state, pos) = self.emit_comments_before_token(token, pos, context_node, flags);
        state.comment_state = comment_state;
        state.source_map_state = self.emit_source_maps_before_token(token, pos, context_node, flags);
        (state, pos)
    }

    pub fn exit_token(
        &mut self,
        token: SyntaxKind,
        pos: i32,
        context_node: &Node,
        previous_state: PrinterState39k06,
    ) { ::tsox_core::fntrace::enter("exit_token"); 
        self.emit_source_maps_after_token(token, pos, context_node, previous_state.source_map_state);
        self.emit_comments_after_token(token, pos, context_node, previous_state.comment_state);
    }
}

pub fn format_synthesized_comment(comment: &SynthesizedComment) -> String { ::tsox_core::fntrace::enter("format_synthesized_comment"); 
    if comment.kind == SyntaxKind::MultiLineCommentTrivia {
        return format!("/*{}*/", comment.text);
    }
    format!("//{}", comment.text)
}

pub fn get_opening_bracket(format: ListFormat) -> &'static str { ::tsox_core::fntrace::enter("get_opening_bracket"); 
    match format & LF_BRACKETS_MASK {
        LF_BRACES => "{",
        LF_PARENTHESIS => "(",
        LF_ANGLE_BRACKETS => "<",
        LF_SQUARE_BRACKETS => "[",
        _ => panic!("Unexpected bracket: {}", format & LF_BRACKETS_MASK),
    }
}

pub fn get_closing_bracket(format: ListFormat) -> &'static str { ::tsox_core::fntrace::enter("get_closing_bracket"); 
    match format & LF_BRACKETS_MASK {
        LF_BRACES => "}",
        LF_PARENTHESIS => ")",
        LF_ANGLE_BRACKETS => ">",
        LF_SQUARE_BRACKETS => "]",
        _ => panic!("Unexpected bracket: {}", format & LF_BRACKETS_MASK),
    }
}
