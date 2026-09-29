#![allow(unused_imports)]

use crate::checker::grammarchecks::*;
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::Diagnostic;

impl Checker {
    // Go checkGrammarAwaitOrAwaitUsing（AwaitExpression 形态）非 AwaitContext
    // 且非顶层上下文：TS1308 + TS1356 related；返回是否已按函数内分支处理
    pub(crate) fn check_await_expression_in_non_async_context(&mut self, node: &Arc<Node>) -> bool {
        if node.flags.contains(NodeFlags::AwaitContext) {
            return false;
        }
        if self.node_is_in_top_level_context(node) {
            return false;
        }
        let file = match self.get_source_file_of_node(node) {
            Some(f) => f,
            None => return false,
        };
        if file.has_parse_diagnostics {
            return false;
        }
        let span = token_range_at(&file.text, node.loc.pos());
        let mut diagnostic = Diagnostic::new(
            Some(Arc::clone(&file)),
            span,
            X_AWAIT_EXPRESSIONS_ARE_ONLY_ALLOWED_WITHIN_ASYNC_FUNCTIONS_AND_AT_THE_TOP_LEVELS_OF_MODULES,
            Vec::new(),
        );
        let container = crate::checker::utilities_get_assignment_target::get_containing_function_or_class_static_block(node);
        if let Some(c) = &container {
            if c.kind != SyntaxKind::Constructor && !Self::node_has_async_modifier(c) {
                push_async_hint_related(&mut diagnostic, &file, c);
            }
        }
        self.diagnostics.add(diagnostic);
        true
    }

    // Go checkGrammarForInOrForOfStatement 的 for-await 分支：
    // 非 AwaitContext 时按顶层/函数内分别报 TS1375/TS1378/TS1103
    pub(crate) fn check_for_await_out_of_context(&mut self, node: &Arc<Node>) -> bool {
        let tsox_frontend::ast::NodeData::ForInOrOfStatement(data) = &node.data else {
            return false;
        };
        let Some(await_modifier) = &data.await_modifier else {
            return false;
        };
        if node.flags.contains(NodeFlags::AwaitContext) {
            return false;
        }
        let file = match self.get_source_file_of_node(node) {
            Some(f) => f,
            None => return false,
        };
        if file.has_parse_diagnostics {
            return false;
        }
        if self.node_is_in_top_level_context(node) {
            if !tsox_frontend::ast::is_external_module(&file) {
                self.diagnostics.add(Diagnostic::new(
                    Some(Arc::clone(&file)),
                    await_modifier.loc,
                    X_FOR_AWAIT_LOOPS_ARE_ONLY_ALLOWED_AT_THE_TOP_LEVEL_OF_A_FILE_WHEN_THAT_FILE_IS_A_MODULE_BUT_THIS_FILE_HAS_NO_IMPORTS_OR_EXPORTS_CONSIDER_ADDING_AN_EMPTY_EXPORT_TO_MAKE_THIS_FILE_A_MODULE,
                    Vec::new(),
                ));
            }
            let module_kind = self.compiler_options.get_emit_module_kind();
            let module_ok = matches!(
                module_kind,
                tsox_core::core::compiler_options::ModuleKind::ES2022
                    | tsox_core::core::compiler_options::ModuleKind::ESNext
                    | tsox_core::core::compiler_options::ModuleKind::System
                    | tsox_core::core::compiler_options::ModuleKind::Preserve
            );
            let target_ok = self.compiler_options.target
                >= tsox_core::core::compiler_options::ScriptTarget::ES2017;
            if !(module_ok && target_ok) {
                self.diagnostics.add(Diagnostic::new(
                    Some(Arc::clone(&file)),
                    await_modifier.loc,
                    TOP_LEVEL_FOR_AWAIT_LOOPS_ARE_ONLY_ALLOWED_WHEN_THE_MODULE_OPTION_IS_SET_TO_ES2022_ESNEXT_SYSTEM_NODE16_NODE18_NODE20_NODENEXT_OR_PRESERVE_AND_THE_TARGET_OPTION_IS_SET_TO_ES2017_OR_HIGHER,
                    Vec::new(),
                ));
            }
            return false;
        }
        let mut diagnostic = Diagnostic::new(
            Some(Arc::clone(&file)),
            await_modifier.loc,
            X_FOR_AWAIT_LOOPS_ARE_ONLY_ALLOWED_WITHIN_ASYNC_FUNCTIONS_AND_AT_THE_TOP_LEVELS_OF_MODULES,
            Vec::new(),
        );
        let container = crate::checker::utilities_get_assignment_target::get_containing_function_or_class_static_block(node);
        if let Some(c) = &container {
            if c.kind != SyntaxKind::Constructor
                && c.kind != SyntaxKind::ClassStaticBlockDeclaration
            {
                push_async_hint_related(&mut diagnostic, &file, c);
            }
        }
        self.diagnostics.add(diagnostic);
        true
    }

    // Go ast.IsInTopLevelContext = GetThisContainer(includeArrowFunctions)
    // 为 SourceFile
    fn node_is_in_top_level_context(&self, node: &Arc<Node>) -> bool {
        crate::checker::checker_this_container::get_this_container(node, true, false).kind
            == SyntaxKind::SourceFile
    }

    fn node_has_async_modifier(node: &Arc<Node>) -> bool {
        node.modifiers()
            .as_ref()
            .is_some_and(|m| m.flags().contains(ModifierFlags::Async))
    }
}

// Go scanner.GetRangeOfTokenAtPosition
fn token_range_at(text: &str, pos: usize) -> tsox_core::core::text::TextRange {
    let start = pos.min(text.len());
    let mut scanner = tsox_frontend::scanner::Scanner::new(&text[start..]);
    scanner.scan();
    tsox_core::core::text::TextRange::new(
        start + scanner.token_pos(),
        start + scanner.token_end(),
    )
}

fn push_async_hint_related(
    diagnostic: &mut Diagnostic,
    file: &Arc<tsox_frontend::ast::SourceFile>,
    container: &Arc<Node>,
) {
    let range = error_range_for_container(&file.text, container);
    diagnostic.related_information.push(Diagnostic::new(
        Some(Arc::clone(file)),
        range,
        DID_YOU_MEAN_TO_MARK_THIS_FUNCTION_AS_ASYNC,
        Vec::new(),
    ));
}

// Go scanner.GetErrorRangeForNode 的 function-like 子集：
// 声明类取名字（含赋值名回退），箭头函数按块体首行截断
fn error_range_for_container(text: &str, node: &Arc<Node>) -> tsox_core::core::text::TextRange {
    if node.kind == SyntaxKind::ArrowFunction {
        let pos = skip_trivia(text, node.loc.pos());
        if let tsox_frontend::ast::NodeData::ArrowFunction(d) = &node.data {
            {
                let body = &d.body;
                if body.kind == SyntaxKind::Block {
                    let start_line = line_index_of(text, body.loc.pos());
                    let end_line = line_index_of(text, body.end());
                    if start_line < end_line {
                        let line_end = end_of_line(text, start_line);
                        return tsox_core::core::text::TextRange::new(pos, line_end);
                    }
                }
            }
        }
        return tsox_core::core::text::TextRange::new(pos, node.end());
    }
    match tsox_frontend::ast::utilities::get_name_of_declaration(node) {
        Some(name) => {
            tsox_core::core::text::TextRange::new(skip_trivia(text, name.loc.pos()), name.end())
        }
        None => token_range_at(text, node.loc.pos()),
    }
}

fn line_index_of(text: &str, pos: usize) -> usize {
    text[..pos.min(text.len())].matches('\n').count()
}

fn end_of_line(text: &str, line: usize) -> usize {
    let mut rest = text;
    let mut offset = 0;
    for _ in 0..line {
        match rest.find('\n') {
            Some(i) => {
                offset += i + 1;
                rest = &rest[i + 1..];
            }
            None => return text.len(),
        }
    }
    let line_len = rest.find('\n').unwrap_or(rest.len());
    let line_end = offset + line_len;
    let mut trimmed = line_end;
    let bytes = text.as_bytes();
    while trimmed > offset
        && (bytes[trimmed - 1] == b'\r'
            || bytes[trimmed - 1] == b'\n'
            || bytes[trimmed - 1] == b' ')
    {
        trimmed -= 1;
    }
    trimmed + 1
}

fn skip_trivia(text: &str, pos: usize) -> usize {
    let bytes = text.as_bytes();
    let mut i = pos.min(bytes.len());
    while i < bytes.len() {
        match bytes[i] {
            b' ' | b'\t' | b'\x0b' | b'\x0c' | b'\r' | b'\n' => i += 1,
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(bytes.len());
            }
            _ => break,
        }
    }
    i
}
