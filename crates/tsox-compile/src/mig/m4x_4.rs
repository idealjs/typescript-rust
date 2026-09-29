#![allow(unused_imports, dead_code, unused_variables)]

use std::sync::{Arc, Mutex, Once};

use tsox_core::collections::set::Set;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::tristate::Tristate;
use tsox_core::tspath::{self, Path};
use tsox_tsoptions::mig::m5h_3::SourceOutputAndProjectReference;
use tsox_tsoptions::tsoptions::ParsedCommandLine;

use crate::compiler::{Program, SourceFile};
use super::m4x_2::ProjectReferenceFileMapper;

pub struct LazyValue<T> {
    value: Option<T>,
    once: Once,
    initialized: std::sync::atomic::AtomicBool,
}

impl<T> Default for LazyValue<T> {
    fn default() -> Self {
        Self {
            value: None,
            once: Once::new(),
            initialized: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

impl<T> LazyValue<T> {
    pub fn get_value(&mut self, compute: impl FnOnce() -> T) -> &T {
        self.once.call_once(|| {
            if self.value.is_none() {
                self.value = Some(compute());
            }
            self.initialized
                .store(true, std::sync::atomic::Ordering::SeqCst);
        });
        self.value.as_ref().unwrap()
    }

    pub fn try_reuse(&mut self, from: &LazyValue<T>) {
        if from.initialized.load(std::sync::atomic::Ordering::SeqCst) {
            self.value = unsafe { std::ptr::read(&from.value) };
            self.initialized
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }
}

pub fn has_zero_or_one_asterisk_character(s: &str) -> bool {
    let mut seen_asterisk = false;
    for ch in s.chars() {
        if ch == '*' {
            if !seen_asterisk {
                seen_asterisk = true;
            } else {
                return false;
            }
        }
    }
    true
}

pub fn module_resolution_supports_package_json_exports_and_imports(
    module_resolution: tsox_core::core::compiler_options_kinds::ModuleResolutionKind,
) -> bool {
    let module_resolution = module_resolution as i32;
    (module_resolution >= tsox_core::core::compiler_options_kinds::ModuleResolutionKind::Node16 as i32
        && module_resolution
            <= tsox_core::core::compiler_options_kinds::ModuleResolutionKind::NodeNext as i32)
        || module_resolution
            == tsox_core::core::compiler_options_kinds::ModuleResolutionKind::Bundler as i32
}

impl Program {
    pub(crate) fn project_reference_file_mapper(&self) -> &ProjectReferenceFileMapper {
        self.project_reference_file_mapper.as_ref()
    }

    pub fn to_path(&self, filename: &str) -> Path {
        tspath::to_path(
            filename,
            self.get_current_directory(),
            self.use_case_sensitive_file_names(),
        )
    }

    pub fn init_checker_pool(self: &Arc<Self>) {
        if !self.finished_processing {
            panic!("Program must finish processing files before initializing checker pool");
        }
        let pool: Arc<dyn crate::mig::m4u_2::CheckerPool> = match self.opts.create_checker_pool.as_ref()
        {
            Some(create) => create(self),
            None => Arc::new(super::m4u_2::new_checker_pool_with_tracing(
                Arc::clone(self),
                self.tracing.clone(),
            )),
        };
        self.checker_pool.set(pool.clone()).ok();
        self.compiler_checker_pool.set(pool).ok();
    }

    pub fn needs_import_helpers_import_specifier(&self, file: &Arc<SourceFile>) -> bool {
        let (redirect, _) = self
            .project_reference_file_mapper()
            .get_redirect_for_resolution(file);
        let options_for_file = super::m4x_2::get_compiler_options_with_redirect(
            &self.options(),
            redirect.as_deref(),
        );
        if options_for_file.import_helpers != Tristate::True {
            return false;
        }
        let is_javascript_file = tsox_frontend::ast::is_source_file_js(file);
        let is_external_module_file = tsox_frontend::ast::is_external_module(file);
        if !is_javascript_file
            && (file.is_declaration_file
                || (!options_for_file.get_isolated_modules() && !is_external_module_file))
        {
            return false;
        }
        true
    }

    pub fn jsx_runtime_import_specifier(&self, file: &Arc<SourceFile>) -> String {
        if !tsox_frontend::ast::is_source_file_js(file)
            && file.script_kind != tsox_frontend::ast::ScriptKind::Tsx
        {
            return String::new();
        }
        let (redirect, _) = self
            .project_reference_file_mapper()
            .get_redirect_for_resolution(file);
        let options_for_file = super::m4x_2::get_compiler_options_with_redirect(
            &self.options(),
            redirect.as_deref(),
        );
        tsox_emit::mig::m4f::r33k11_defs::get_jsx_runtime_import(
            &tsox_emit::mig::m4f::r33k11_defs::get_jsx_implicit_import_base(&options_for_file, file),
            &options_for_file,
        )
    }

    pub fn get_source_files_to_emit(
        &self,
        target_source_files: Option<&[Arc<SourceFile>]>,
        force_dts_emit: bool,
        force_js_emit: bool,
    ) -> Vec<Arc<SourceFile>> {
        super::m4v::get_source_files_to_emit(
            self,
            target_source_files.map(|files| files.to_vec()),
            force_dts_emit,
            force_js_emit,
        )
    }

    pub fn get_mode_for_type_reference_directive_in_file(
        &self,
        r: &tsox_frontend::ast::node_source_file::FileReference,
        source_file: &Arc<SourceFile>,
    ) -> tsox_core::core::compiler_options_kinds::ResolutionMode {
        if r.resolution_mode != tsox_core::core::compiler_options_kinds::ResolutionMode::None {
            return r.resolution_mode;
        }
        self.get_default_resolution_mode_for_file(source_file)
    }

    pub fn get_semantic_diagnostics_with_checker(
        &self,
        c: &tsox_checker::checker::Checker,
        source_file: &Arc<SourceFile>,
    ) -> Vec<Arc<tsox_frontend::ast::Diagnostic>> {
        let mut result = super::m4w_4::filter_no_emit_semantic_diagnostics(
            self.get_bind_and_check_diagnostics_with_checker(c, source_file),
            &self.options(),
        );
        result.extend(self.get_include_processor_diagnostics(source_file));
        result
    }

    pub fn get_bind_and_check_diagnostics_with_checker(
        &self,
        file_checker: &tsox_checker::checker::Checker,
        source_file: &Arc<SourceFile>,
    ) -> Vec<Arc<tsox_frontend::ast::Diagnostic>> {
        let compiler_options = self.options();
        if self.skip_type_checking(source_file, false) {
            return Vec::new();
        }

        let mut diags = source_file.bind_diagnostics();
        diags.extend(
            file_checker
                .get_diagnostics()
                .get_for_file(&source_file.file_name)
                .into_iter()
                .map(Arc::new),
        );

        let is_plain_js =
            tsox_frontend::ast::mig::m3g_2::is_plain_js_file(Some(source_file), compiler_options.check_js);
        if is_plain_js {
            return diags
                .into_iter()
                .filter(|d| plain_js_errors_has(d.code()))
                .collect();
        }

        let is_js = source_file.script_kind == tsox_frontend::ast::ScriptKind::Js
            || source_file.script_kind == tsox_frontend::ast::ScriptKind::Jsx;
        let is_check_js =
            is_js && tsox_frontend::ast::mig::m3f_4::is_check_js_enabled_for_file(source_file, &compiler_options);
        if is_check_js {
            diags.extend(
                tsox_frontend::ast::mig::m3b_2::jsdoc_diagnostics(source_file)
                    .iter()
                    .cloned()
                    .map(Arc::new),
            );
        }

        let (mut filtered, directives_by_line) =
            self.get_diagnostics_with_preceding_directives(source_file, diags);
        for directive in directives_by_line.values() {
            if directive.kind == tsox_frontend::scanner::CommentDirectiveKind::ExpectError {
                filtered.push(Arc::new(tsox_frontend::ast::Diagnostic::new(
                    filtered.last().and_then(|d| d.file.clone()),
                    tsox_core::core::text::TextRange::new(directive.pos, directive.end),
                    tsox_core::diagnostics::messages_generated::UNUSED_TS_EXPECT_ERROR_DIRECTIVE,
                    Vec::new(),
                )));
            }
        }
        apply_content_mapper_diagnostic_directives(source_file, filtered)
    }

    pub fn get_diagnostics_with_preceding_directives(
        &self,
        source_file: &Arc<SourceFile>,
        diags: Vec<Arc<tsox_frontend::ast::Diagnostic>>,
    ) -> (
        Vec<Arc<tsox_frontend::ast::Diagnostic>>,
        std::collections::HashMap<usize, tsox_frontend::scanner::CommentDirective>,
    ) {
        let mut directives_by_line = std::collections::HashMap::new();
        if source_file.comment_directives.is_empty() {
            return (diags, directives_by_line);
        }
        for directive in &source_file.comment_directives {
            let line = tsox_emit::mig::m4m_2::get_ecma_line_of_position(
                source_file,
                directive.pos,
            );
            directives_by_line.insert(line, directive.clone());
        }
        let line_starts =
            tsox_frontend::format::mig::m4t_3::get_ecma_line_starts(source_file);
        let mut filtered = Vec::with_capacity(diags.len());
        for diagnostic in diags {
            let mut ignore_diagnostic = false;
            let start_line = tsox_frontend::scanner::mig::w1::compute_line_of_position(
                &line_starts,
                diagnostic.pos() as i32,
            );
            let mut line = start_line.saturating_sub(1);
            while line != usize::MAX {
                if let Some(directive) = directives_by_line.get_mut(&line) {
                    ignore_diagnostic = true;
                    directive.kind = tsox_frontend::scanner::CommentDirectiveKind::Ignore;
                    break;
                }
                if !is_comment_or_blank_line(
                    &source_file.text,
                    line_starts[line] as usize,
                ) {
                    break;
                }
                line -= 1;
            }
            if !ignore_diagnostic {
                filtered.push(diagnostic);
            }
        }
        (filtered, directives_by_line)
    }

    pub fn get_declaration_diagnostics_for_file(
        self: &Arc<Self>,
        source_file: &Arc<SourceFile>,
    ) -> Vec<Arc<tsox_frontend::ast::Diagnostic>> {
        if source_file.is_declaration_file {
            return Vec::new();
        }
        let (host, done) = super::m4u_3::new_emit_host(Arc::clone(self), source_file);
        let diagnostics =
            super::m4v::get_declaration_diagnostics(Arc::new(host), source_file);
        drop(done);
        diagnostics.into_iter().map(Arc::new).collect()
    }

    pub fn get_suggestion_diagnostics_with_checker(
        &self,
        file_checker: &tsox_checker::checker::Checker,
        source_file: &Arc<SourceFile>,
    ) -> Vec<Arc<tsox_frontend::ast::Diagnostic>> {
        if self.skip_type_checking(source_file, false) {
            return Vec::new();
        }
        file_checker
            .get_suggestion_diagnostics()
            .get_for_file(&source_file.file_name)
            .into_iter()
            .map(Arc::new)
            .collect()
    }
}

pub fn apply_content_mapper_diagnostic_directives(
    source_file: &Arc<SourceFile>,
    diags: Vec<Arc<tsox_frontend::ast::Diagnostic>>,
) -> Vec<Arc<tsox_frontend::ast::Diagnostic>> {
    let directives = tsox_frontend::ast::mig::m3b_2::diagnostic_directives(source_file);
    if directives.is_empty() {
        return diags;
    }
    let mut used = vec![false; directives.len()];
    let mark_used = |diag: &tsox_frontend::ast::Diagnostic, used: &mut Vec<bool>| -> bool {
        if !diag.source().is_empty() {
            return false;
        }
        for (i, directive) in directives.iter().enumerate() {
            if diag.pos() as usize >= directive.virtual_range.pos()
                && (diag.pos() as usize) < directive.virtual_range.end()
            {
                used[i] = true;
                return true;
            }
        }
        false
    };
    let filtered: Vec<_> = diags
        .into_iter()
        .filter(|diag| !mark_used(diag, &mut used))
        .collect();
    let mut filtered = filtered;
    for (i, directive) in directives.iter().enumerate() {
        if directive.policy == tsox_frontend::ast::mig::m3b_2::MappedDiagnosticDirectivePolicy::EXPECT
            && !used[i]
        {
            filtered.push(Arc::new(tsox_frontend::ast::mig::m3d_2::new_external_diagnostic(
                filtered.last().map(|d| d.file.clone()).flatten(),
                directive.original_range.clone(),
                directive.source.clone(),
                tsox_core::diagnostics::Category::Error,
                directive.unused_code,
                directive.unused_message_text.clone(),
            )));
        }
    }
    filtered
}

pub fn is_comment_or_blank_line(text: &str, pos: usize) -> bool {
    let bytes = text.as_bytes();
    let mut pos = pos;
    while pos < bytes.len() && (bytes[pos] == b' ' || bytes[pos] == b'\t') {
        pos += 1;
    }
    pos == bytes.len()
        || (pos < bytes.len() && (bytes[pos] == b'\r' || bytes[pos] == b'\n'))
        || (pos + 1 < bytes.len() && bytes[pos] == b'/' && bytes[pos + 1] == b'/')
}
static PLAIN_JS_ERRORS: &[tsox_core::diagnostics::Message] = &[
    tsox_core::diagnostics::messages_generated::A_BREAK_STATEMENT_CAN_ONLY_BE_USED_WITHIN_AN_ENCLOSING_ITERATION_OR_SWITCH_STATEMENT,
    tsox_core::diagnostics::messages_generated::A_BREAK_STATEMENT_CAN_ONLY_JUMP_TO_A_LABEL_OF_AN_ENCLOSING_STATEMENT,
    tsox_core::diagnostics::messages_generated::A_CLASS_DECLARATION_WITHOUT_THE_DEFAULT_MODIFIER_MUST_HAVE_A_NAME,
    tsox_core::diagnostics::messages_generated::A_CLASS_MEMBER_CANNOT_HAVE_THE_0_KEYWORD,
    tsox_core::diagnostics::messages_generated::A_COMMA_EXPRESSION_IS_NOT_ALLOWED_IN_A_COMPUTED_PROPERTY_NAME,
    tsox_core::diagnostics::messages_generated::A_CONTINUE_STATEMENT_CAN_ONLY_BE_USED_WITHIN_AN_ENCLOSING_ITERATION_STATEMENT,
    tsox_core::diagnostics::messages_generated::A_CONTINUE_STATEMENT_CAN_ONLY_JUMP_TO_A_LABEL_OF_AN_ENCLOSING_ITERATION_STATEMENT,
    tsox_core::diagnostics::messages_generated::A_DEFAULT_CLAUSE_CANNOT_APPEAR_MORE_THAN_ONCE_IN_A_SWITCH_STATEMENT,
    tsox_core::diagnostics::messages_generated::A_DEFAULT_EXPORT_MUST_BE_AT_THE_TOP_LEVEL_OF_A_FILE_OR_MODULE_DECLARATION,
    tsox_core::diagnostics::messages_generated::A_DEFINITE_ASSIGNMENT_ASSERTION_IS_NOT_PERMITTED_IN_THIS_CONTEXT,
    tsox_core::diagnostics::messages_generated::A_DESTRUCTURING_DECLARATION_MUST_HAVE_AN_INITIALIZER,
    tsox_core::diagnostics::messages_generated::A_GET_ACCESSOR_CANNOT_HAVE_PARAMETERS,
    tsox_core::diagnostics::messages_generated::A_LABEL_IS_NOT_ALLOWED_HERE,
    tsox_core::diagnostics::messages_generated::A_MODULE_CANNOT_HAVE_MULTIPLE_DEFAULT_EXPORTS,
    tsox_core::diagnostics::messages_generated::AN_EXPORT_DECLARATION_CANNOT_HAVE_MODIFIERS,
    tsox_core::diagnostics::messages_generated::AN_EXPORT_DECLARATION_CAN_ONLY_BE_USED_AT_THE_TOP_LEVEL_OF_A_MODULE,
    tsox_core::diagnostics::messages_generated::AN_IMPORT_DECLARATION_CANNOT_HAVE_MODIFIERS,
    tsox_core::diagnostics::messages_generated::AN_IMPORT_DECLARATION_CAN_ONLY_BE_USED_AT_THE_TOP_LEVEL_OF_A_MODULE,
    tsox_core::diagnostics::messages_generated::AN_OBJECT_MEMBER_CANNOT_BE_DECLARED_OPTIONAL,
    tsox_core::diagnostics::messages_generated::ANOTHER_EXPORT_DEFAULT_IS_HERE,
    tsox_core::diagnostics::messages_generated::A_REST_ELEMENT_CANNOT_CONTAIN_A_BINDING_PATTERN,
    tsox_core::diagnostics::messages_generated::A_REST_ELEMENT_CANNOT_HAVE_AN_INITIALIZER,
    tsox_core::diagnostics::messages_generated::A_REST_ELEMENT_CANNOT_HAVE_A_PROPERTY_NAME,
    tsox_core::diagnostics::messages_generated::A_REST_ELEMENT_MUST_BE_LAST_IN_A_DESTRUCTURING_PATTERN,
    tsox_core::diagnostics::messages_generated::A_REST_PARAMETER_CANNOT_HAVE_AN_INITIALIZER,
    tsox_core::diagnostics::messages_generated::A_REST_PARAMETER_MUST_BE_LAST_IN_A_PARAMETER_LIST,
    tsox_core::diagnostics::messages_generated::A_REST_PARAMETER_OR_BINDING_PATTERN_MAY_NOT_HAVE_A_TRAILING_COMMA,
    tsox_core::diagnostics::messages_generated::A_RETURN_STATEMENT_CANNOT_BE_USED_INSIDE_A_CLASS_STATIC_BLOCK,
    tsox_core::diagnostics::messages_generated::ARGUMENT_OF_DYNAMIC_IMPORT_CANNOT_BE_SPREAD_ELEMENT,
    tsox_core::diagnostics::messages_generated::A_SET_ACCESSOR_CANNOT_HAVE_REST_PARAMETER,
    tsox_core::diagnostics::messages_generated::A_SET_ACCESSOR_MUST_HAVE_EXACTLY_ONE_PARAMETER,
    tsox_core::diagnostics::messages_generated::CANNOT_ASSIGN_TO_PRIVATE_METHOD_0_PRIVATE_METHODS_ARE_NOT_WRITABLE,
    tsox_core::diagnostics::messages_generated::CANNOT_REDECLARE_BLOCK_SCOPED_VARIABLE_0,
    tsox_core::diagnostics::messages_generated::CANNOT_REDECLARE_IDENTIFIER_0_IN_CATCH_CLAUSE,
    tsox_core::diagnostics::messages_generated::CATCH_CLAUSE_VARIABLE_CANNOT_HAVE_AN_INITIALIZER,
    tsox_core::diagnostics::messages_generated::CLASS_CONSTRUCTOR_MAY_NOT_BE_A_GENERATOR,
    tsox_core::diagnostics::messages_generated::CLASS_CONSTRUCTOR_MAY_NOT_BE_AN_ACCESSOR,
    tsox_core::diagnostics::messages_generated::CLASS_DECORATORS_CAN_T_BE_USED_WITH_STATIC_PRIVATE_IDENTIFIER_CONSIDER_REMOVING_THE_EXPERIMENTAL_DECORATOR,
    tsox_core::diagnostics::messages_generated::CLASSES_CAN_ONLY_EXTEND_A_SINGLE_CLASS,
    tsox_core::diagnostics::messages_generated::CLASSES_MAY_NOT_HAVE_A_FIELD_NAMED_CONSTRUCTOR,
    tsox_core::diagnostics::messages_generated::CODE_CONTAINED_IN_A_CLASS_IS_EVALUATED_IN_JAVASCRIPT_S_STRICT_MODE_WHICH_DOES_NOT_ALLOW_THIS_USE_OF_0_FOR_MORE_INFORMATION_SEE_HTTPS_COLON_SLASH_SLASHDEVELOPER_MOZILLA_ORG_SLASHEN_US_SLASHDOCS_SLASHWEB_SLASHJAVASCRIPT_SLASHREFERENCE_SLASHSTRICT_MODE,
    tsox_core::diagnostics::messages_generated::DID_YOU_MEAN_TO_USE_A_COLON_AN_CAN_ONLY_FOLLOW_A_PROPERTY_NAME_WHEN_THE_CONTAINING_OBJECT_LITERAL_IS_PART_OF_A_DESTRUCTURING_PATTERN,
    tsox_core::diagnostics::messages_generated::DUPLICATE_LABEL_0,
    tsox_core::diagnostics::messages_generated::DYNAMIC_IMPORTS_CAN_ONLY_ACCEPT_A_MODULE_SPECIFIER_AND_AN_OPTIONAL_SET_OF_ATTRIBUTES_AS_ARGUMENTS,
    tsox_core::diagnostics::messages_generated::IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_AT_THE_TOP_LEVEL_OF_A_MODULE,
    tsox_core::diagnostics::messages_generated::IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE_MODULES_ARE_AUTOMATICALLY_IN_STRICT_MODE,
    tsox_core::diagnostics::messages_generated::IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_THAT_CANNOT_BE_USED_HERE,
    tsox_core::diagnostics::messages_generated::INVALID_USE_OF_0_IN_STRICT_MODE,
    tsox_core::diagnostics::messages_generated::INVALID_USE_OF_0_MODULES_ARE_AUTOMATICALLY_IN_STRICT_MODE,
    tsox_core::diagnostics::messages_generated::JSX_ATTRIBUTES_MUST_ONLY_BE_ASSIGNED_A_NON_EMPTY_EXPRESSION,
    tsox_core::diagnostics::messages_generated::JSX_ELEMENTS_CANNOT_HAVE_MULTIPLE_ATTRIBUTES_WITH_THE_SAME_NAME,
    tsox_core::diagnostics::messages_generated::JSX_EXPRESSIONS_MAY_NOT_USE_THE_COMMA_OPERATOR_DID_YOU_MEAN_TO_WRITE_AN_ARRAY,
    tsox_core::diagnostics::messages_generated::JSX_PROPERTY_ACCESS_EXPRESSIONS_CANNOT_INCLUDE_JSX_NAMESPACE_NAMES,
    tsox_core::diagnostics::messages_generated::JUMP_TARGET_CANNOT_CROSS_FUNCTION_BOUNDARY,
    tsox_core::diagnostics::messages_generated::LINE_TERMINATOR_NOT_PERMITTED_BEFORE_ARROW,
    tsox_core::diagnostics::messages_generated::MODIFIERS_CANNOT_APPEAR_HERE,
    tsox_core::diagnostics::messages_generated::ONLY_A_SINGLE_VARIABLE_DECLARATION_IS_ALLOWED_IN_A_FOR_IN_STATEMENT,
    tsox_core::diagnostics::messages_generated::ONLY_A_SINGLE_VARIABLE_DECLARATION_IS_ALLOWED_IN_A_FOR_OF_STATEMENT,
    tsox_core::diagnostics::messages_generated::PRIVATE_FIELD_0_MUST_BE_DECLARED_IN_AN_ENCLOSING_CLASS,
    tsox_core::diagnostics::messages_generated::PRIVATE_IDENTIFIERS_ARE_NOT_ALLOWED_OUTSIDE_CLASS_BODIES,
    tsox_core::diagnostics::messages_generated::PRIVATE_IDENTIFIERS_ARE_ONLY_ALLOWED_IN_CLASS_BODIES_AND_MAY_ONLY_BE_USED_AS_PART_OF_A_CLASS_MEMBER_DECLARATION_PROPERTY_ACCESS_OR_ON_THE_LEFT_HAND_SIDE_OF_AN_IN_EXPRESSION,
    tsox_core::diagnostics::messages_generated::PROPERTY_0_IS_NOT_ACCESSIBLE_OUTSIDE_CLASS_1_BECAUSE_IT_HAS_A_PRIVATE_IDENTIFIER,
    tsox_core::diagnostics::messages_generated::TAGGED_TEMPLATE_EXPRESSIONS_ARE_NOT_PERMITTED_IN_AN_OPTIONAL_CHAIN,
    tsox_core::diagnostics::messages_generated::THE_FIRST_EXPORT_DEFAULT_IS_HERE,
    tsox_core::diagnostics::messages_generated::THE_LEFT_HAND_SIDE_OF_A_FOR_OF_STATEMENT_MAY_NOT_BE_ASYNC,
    tsox_core::diagnostics::messages_generated::THE_VARIABLE_DECLARATION_OF_A_FOR_IN_STATEMENT_CANNOT_HAVE_AN_INITIALIZER,
    tsox_core::diagnostics::messages_generated::THE_VARIABLE_DECLARATION_OF_A_FOR_OF_STATEMENT_CANNOT_HAVE_AN_INITIALIZER,
    tsox_core::diagnostics::messages_generated::THIS_CONDITION_WILL_ALWAYS_RETURN_0_SINCE_JAVASCRIPT_COMPARES_OBJECTS_BY_REFERENCE_NOT_VALUE,
    tsox_core::diagnostics::messages_generated::TRAILING_COMMA_NOT_ALLOWED,
    tsox_core::diagnostics::messages_generated::VARIABLE_DECLARATION_LIST_CANNOT_BE_EMPTY,
    tsox_core::diagnostics::messages_generated::X_0_AND_1_OPERATIONS_CANNOT_BE_MIXED_WITHOUT_PARENTHESES,
    tsox_core::diagnostics::messages_generated::X_0_DECLARATIONS_CAN_ONLY_BE_DECLARED_INSIDE_A_BLOCK,
    tsox_core::diagnostics::messages_generated::X_0_DECLARATIONS_MUST_BE_INITIALIZED,
    tsox_core::diagnostics::messages_generated::X_0_EXPECTED,
    tsox_core::diagnostics::messages_generated::X_0_IS_NOT_A_VALID_META_PROPERTY_FOR_KEYWORD_1_DID_YOU_MEAN_2,
    tsox_core::diagnostics::messages_generated::X_0_LIST_CANNOT_BE_EMPTY,
    tsox_core::diagnostics::messages_generated::X_0_MODIFIER_ALREADY_SEEN,
    tsox_core::diagnostics::messages_generated::X_0_MODIFIER_CANNOT_APPEAR_ON_A_CONSTRUCTOR_DECLARATION,
    tsox_core::diagnostics::messages_generated::X_0_MODIFIER_CANNOT_APPEAR_ON_A_MODULE_OR_NAMESPACE_ELEMENT,
    tsox_core::diagnostics::messages_generated::X_0_MODIFIER_CANNOT_APPEAR_ON_A_PARAMETER,
    tsox_core::diagnostics::messages_generated::X_0_MODIFIER_CANNOT_APPEAR_ON_CLASS_ELEMENTS_OF_THIS_KIND,
    tsox_core::diagnostics::messages_generated::X_0_MODIFIER_CANNOT_BE_USED_HERE,
    tsox_core::diagnostics::messages_generated::X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
    tsox_core::diagnostics::messages_generated::X_AWAIT_EXPRESSIONS_ARE_ONLY_ALLOWED_WITHIN_ASYNC_FUNCTIONS_AND_AT_THE_TOP_LEVELS_OF_MODULES,
    tsox_core::diagnostics::messages_generated::X_AWAIT_USING_STATEMENTS_ARE_ONLY_ALLOWED_WITHIN_ASYNC_FUNCTIONS_AND_AT_THE_TOP_LEVELS_OF_MODULES,
    tsox_core::diagnostics::messages_generated::X_CONSTRUCTOR_IS_A_RESERVED_WORD,
    tsox_core::diagnostics::messages_generated::X_DELETE_CANNOT_BE_CALLED_ON_AN_IDENTIFIER_IN_STRICT_MODE,
    tsox_core::diagnostics::messages_generated::X_EXTENDS_CLAUSE_ALREADY_SEEN,
    tsox_core::diagnostics::messages_generated::X_FOR_AWAIT_LOOPS_CANNOT_BE_USED_INSIDE_A_CLASS_STATIC_BLOCK,
    tsox_core::diagnostics::messages_generated::X_LET_IS_NOT_ALLOWED_TO_BE_USED_AS_A_NAME_IN_LET_OR_CONST_DECLARATIONS,
    tsox_core::diagnostics::messages_generated::X_WITH_STATEMENTS_ARE_NOT_ALLOWED_IN_STRICT_MODE,
];

pub(crate) fn plain_js_errors_has(code: i32) -> bool {
    PLAIN_JS_ERRORS.iter().any(|m| m.code == code)
}
