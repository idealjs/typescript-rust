#![allow(unused_imports)]

use crate::checker::checker::*;
use crate::checker::grammarchecks::*;
use tsox_core::core::compiler_options::ModuleKind;
use tsox_core::core::compiler_options::ScriptTarget;

impl Checker {
    /// Go checkGrammarAwaitOrAwaitUsing（declaration list 形态）：class static block
    /// 禁用；容器非 async 报 TS2852；顶层 await using 按 module/target 门槛报 TS2854
    pub(crate) fn check_grammar_await_or_await_using_declaration_list(
        &mut self,
        node: &Arc<Node>,
    ) -> bool {
        let container = crate::checker::utilities_get_assignment_target::
            get_containing_function_or_class_static_block(node);
        let Some(container) = container else {
            return self.check_top_level_await_using_thresholds(node);
        };
        if container.kind == SyntaxKind::ClassStaticBlockDeclaration {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                node.loc,
                tsox_core::diagnostics::messages_generated::
                    X_AWAIT_USING_STATEMENTS_CANNOT_BE_USED_INSIDE_A_CLASS_STATIC_BLOCK,
                Vec::new(),
            ));
            return true;
        }
        if container.has_syntactic_modifier(ModifierFlags::Async) {
            return false;
        }
        let source_file = self.current_file.clone();
        if source_file.as_ref().is_some_and(|f| f.has_parse_diagnostics) {
            return false;
        }
        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
            source_file,
            node.loc,
            tsox_core::diagnostics::messages_generated::
                X_AWAIT_USING_STATEMENTS_ARE_ONLY_ALLOWED_WITHIN_ASYNC_FUNCTIONS_AND_AT_THE_TOP_LEVELS_OF_MODULES,
            Vec::new(),
        ));
        true
    }

    fn check_top_level_await_using_thresholds(&mut self, node: &Arc<Node>) -> bool {
        let Some(source_file) = self.current_file.clone() else {
            return false;
        };
        if source_file.has_parse_diagnostics {
            return false;
        }
        let module_kind = self.module_kind;
        let target = self.compiler_options.target;
        let allowed_module = matches!(
            module_kind,
            ModuleKind::ES2022
                | ModuleKind::ESNext
                | ModuleKind::Preserve
                | ModuleKind::System
                | ModuleKind::Node16
                | ModuleKind::Node18
                | ModuleKind::Node20
                | ModuleKind::NodeNext
        );
        if allowed_module && target >= ScriptTarget::ES2017 {
            return false;
        }
        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
            Some(source_file),
            node.loc,
            tsox_core::diagnostics::messages_generated::
                TOP_LEVEL_AWAIT_USING_STATEMENTS_ARE_ONLY_ALLOWED_WHEN_THE_MODULE_OPTION_IS_SET_TO_ES2022_ESNEXT_SYSTEM_NODE16_NODE18_NODE20_NODENEXT_OR_PRESERVE_AND_THE_TARGET_OPTION_IS_SET_TO_ES2017_OR_HIGHER,
            Vec::new(),
        ));
        true
    }
}
