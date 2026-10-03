#![allow(unused_imports, dead_code, unused_variables)]

use std::collections::HashMap;
use std::sync::Arc;

use tsox_core::collections::set::Set;
use tsox_core::core::tristate::Tristate;
use tsox_core::tspath::{self, Path};
use tsox_core::diagnostics::messages_generated as dg;
use tsox_core::diagnostics::Message;
use tsox_frontend::ast::Diagnostic;
use tsox_frontend::ast::NodeData;
use tsox_frontend::ast::node_node::Node;

use crate::compiler::{Program, SourceFile};
use super::m4v::{
    IncludeExplainingDiagnostic, ProcessingDiagnostic, ProcessingDiagnosticData,
    ProcessingDiagnosticKind,
};
use super::m4x_4::{has_zero_or_one_asterisk_character, module_resolution_supports_package_json_exports_and_imports};
use tsox_core::core::compiler_options_kinds::{ModuleKind, ModuleResolutionKind, ScriptTarget};
use tsox_frontend::ast::mig::m3d_2::new_compiler_diagnostic;
use tsox_tsoptions::mig::m5h_5::create_diagnostic_for_node_in_source_file_or_compiler_diagnostic;
use tsox_tsoptions::mig::m5i2::for_each_property_assignment;
use tsox_tsoptions::tsoptions::ParsedCommandLine;

use tsox_checker::checker::Program as _;

struct OptionDiagCtx<'a> {
    program: &'a mut Program,
    source_file: Option<Arc<SourceFile>>,
    config_file_path: String,
}

impl<'a> OptionDiagCtx<'a> {
    fn new(program: &'a mut Program) -> Self { ::tsox_core::fntrace::enter("new"); 
        let source_file = program
            .opts
            .config
            .config_file
            .as_ref()
            .map(|config_file| config_file.source_file.clone());
        let config_file_path = source_file
            .as_ref()
            .map(|file| file.file_name.clone())
            .unwrap_or_default();
        Self {
            program,
            source_file,
            config_file_path,
        }
    }

    fn get_compiler_options_property_syntax(&self) -> Option<&Node> { ::tsox_core::fntrace::enter("get_compiler_options_property_syntax"); 
        compiler_options_property_initializer(self.source_file.as_deref())
    }

    fn get_compiler_options_object_literal_syntax(&self) -> Option<&Node> { ::tsox_core::fntrace::enter("get_compiler_options_object_literal_syntax"); 
        config_property_object_literal(self.source_file.as_deref(), "compilerOptions")
    }

    fn create_compiler_options_diagnostic(
        &mut self,
        message: Message,
        args: Vec<String>,
    ) -> Arc<Diagnostic> { ::tsox_core::fntrace::enter("create_compiler_options_diagnostic"); 
        let diag = match self.get_compiler_options_property_syntax() {
            Some(property) => match &property.data {
                NodeData::PropertyAssignment(d) => Arc::new(
                    create_diagnostic_for_node_in_source_file_or_compiler_diagnostic(
                        self.source_file.as_deref(),
                        Some(d.name.as_ref()),
                        message,
                        args,
                    ),
                ),
                _ => Arc::new(new_compiler_diagnostic(message, args)),
            },
            None => Arc::new(new_compiler_diagnostic(message, args)),
        };
        diag
    }

    fn create_diagnostic_for_option(
        &mut self,
        on_key: bool,
        option1: &str,
        option2: &str,
        message: Message,
        args: &[String],
    ) -> Arc<Diagnostic> { ::tsox_core::fntrace::enter("create_diagnostic_for_option"); 
        let source_file = self.source_file.as_deref();
        let object_literal = config_property_object_literal(source_file, "compilerOptions");
        let program = &mut *self.program;
        let diag = Self::create_option_diagnostic_in_object_literal_syntax(
            source_file,
            program,
            object_literal,
            on_key,
            option1,
            option2,
            message,
            args,
        );
        match diag {
            Some(diag) => diag,
            None => self.create_compiler_options_diagnostic(message, args.to_vec()),
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn create_option_diagnostic_in_object_literal_syntax(
        source_file: Option<&SourceFile>,
        program: &mut Program,
        object_literal: Option<&Node>,
        on_key: bool,
        key1: &str,
        key2: &str,
        message: Message,
        args: &[String],
    ) -> Option<Arc<Diagnostic>> { ::tsox_core::fntrace::enter("create_option_diagnostic_in_object_literal_syntax"); 
        let diag = for_each_property_assignment(
            object_literal,
            key1,
            Some(key2),
            &mut |property| {
                let node: Option<&Node> = match &property.data {
                    NodeData::PropertyAssignment(d) => {
                        if on_key {
                            Some(d.name.as_ref())
                        } else {
                            Some(d.initializer.as_ref())
                        }
                    }
                    _ => None,
                };
                Some(Arc::new(
                    create_diagnostic_for_node_in_source_file_or_compiler_diagnostic(
                        source_file,
                        node,
                        message,
                        args.to_vec(),
                    ),
                ))
            },
        );
        diag
    }

    fn create_diagnostic_for_option_name(
        &mut self,
        message: Message,
        option1: &str,
        option2: &str,
        args: &[String],
    ) { ::tsox_core::fntrace::enter("create_diagnostic_for_option_name"); 
        let mut new_args = vec![option1.to_string(), option2.to_string()];
        new_args.extend_from_slice(args);
        let diag = self.create_diagnostic_for_option(true, option1, option2, message, &new_args);
        self.program.program_diagnostics.push(diag);
    }

    fn create_option_value_diagnostic(
        &mut self,
        option1: &str,
        message: Message,
        args: &[String],
    ) { ::tsox_core::fntrace::enter("create_option_value_diagnostic"); 
        let diag = self.create_diagnostic_for_option(false, option1, "", message, args);
        self.program.program_diagnostics.push(diag);
    }

    fn create_diagnostic_for_option_paths(
        &mut self,
        on_key: bool,
        key: &str,
        message: Message,
        args: &[String],
    ) -> Option<Arc<Diagnostic>> { ::tsox_core::fntrace::enter("create_diagnostic_for_option_paths"); 
        let source_file = self.source_file.as_deref();
        let paths_object = config_property_object_literal(source_file, "paths");
        let diag = Self::create_option_diagnostic_in_object_literal_syntax(
            source_file,
            &mut *self.program,
            paths_object,
            on_key,
            key,
            "",
            message,
            args,
        );
        let diag = match diag {
            Some(diag) => diag,
            None => self.create_compiler_options_diagnostic(message, args.to_vec()),
        };
        self.program.program_diagnostics.push(diag.clone());
        Some(diag)
    }

    fn create_diagnostic_for_option_path_key_value(
        &mut self,
        key: &str,
        value_index: usize,
        message: Message,
        args: &[String],
    ) -> Option<Arc<Diagnostic>> { ::tsox_core::fntrace::enter("create_diagnostic_for_option_path_key_value"); 
        let source_file = self.source_file.clone();
        let paths_object = config_property_object_literal(source_file.as_deref(), "paths");
        let mut anchored: Option<Arc<Diagnostic>> = None;
        if let Some(paths_object) = paths_object {
            if let Some(array) = config_property_initializer_of(paths_object, key) {
                if tsox_frontend::ast::node_data_generated::is_array_literal_expression(array) {
                    if let NodeData::ArrayLiteralExpression(data) = &array.data {
                        if let Some(element) = data.elements.nodes.get(value_index) {
                            anchored = Some(Arc::new(
                                create_diagnostic_for_node_in_source_file_or_compiler_diagnostic(
                                    source_file.as_deref(),
                                    Some(element.as_ref()),
                                    message,
                                    args.to_vec(),
                                ),
                            ));
                        }
                    }
                }
            }
        }
        match anchored {
            Some(diag) => {
                self.program.program_diagnostics.push(diag.clone());
                Some(diag)
            }
            None => {
                let diag = self.create_compiler_options_diagnostic(message, args.to_vec());
                self.program.program_diagnostics.push(diag.clone());
                Some(diag)
            }
        }
    }

    fn create_removed_option_diagnostic(&mut self, name: &str, value: &str, use_instead: &str) { ::tsox_core::fntrace::enter("create_removed_option_diagnostic"); 
        let (message, args): (Message, Vec<String>) = if value.is_empty() {
            (
                dg::OPTION_0_HAS_BEEN_REMOVED_PLEASE_REMOVE_IT_FROM_YOUR_CONFIGURATION,
                vec![name.to_string()],
            )
        } else {
            (
                dg::OPTION_0_1_HAS_BEEN_REMOVED_PLEASE_REMOVE_IT_FROM_YOUR_CONFIGURATION,
                vec![name.to_string(), value.to_string()],
            )
        };
        let mut diag = self.create_diagnostic_for_option(value.is_empty(), name, "", message, &args);
        if !use_instead.is_empty() {
            let chain = new_compiler_diagnostic(
                dg::USE_0_INSTEAD,
                vec![use_instead.to_string()],
            );
            if let Some(target) = Arc::get_mut(&mut diag) {
                target.add_message_chain(chain);
            }
        }
        self.program.program_diagnostics.push(diag);
    }
}

fn compiler_options_property_initializer<'a>(
    source_file: Option<&'a SourceFile>,
) -> Option<&'a Node> { ::tsox_core::fntrace::enter("compiler_options_property_initializer"); 
    config_property_initializer_of(
        tsox_tsoptions::mig::m5j::get_ts_config_object_literal_expression(source_file)?,
        "compilerOptions",
    )
}

fn config_property_initializer_of<'a>(object_literal: &'a Node, key: &str) -> Option<&'a Node> { ::tsox_core::fntrace::enter("config_property_initializer_of"); 
    let tsox_frontend::ast::NodeData::ObjectLiteralExpression(data) = &object_literal.data else {
        return None;
    };
    for property in &data.properties.nodes {
        let tsox_frontend::ast::NodeData::PropertyAssignment(d) = &property.data else {
            continue;
        };
        if tsox_frontend::ast::mig::m3h::try_get_text_of_property_name(&d.name).as_deref() == Some(key)
        {
            return Some(&d.initializer);
        }
    }
    None
}

fn config_property_object_literal<'a>(
    source_file: Option<&'a SourceFile>,
    key: &str,
) -> Option<&'a Node> { ::tsox_core::fntrace::enter("config_property_object_literal"); 
    let initializer = config_property_initializer_of(
        tsox_tsoptions::mig::m5j::get_ts_config_object_literal_expression(source_file)?,
        key,
    )?;
    if tsox_frontend::ast::node_data_generated::is_object_literal_expression(initializer) {
        Some(initializer)
    } else {
        None
    }
}

impl Program {
    pub fn verify_project_references(&mut self) { ::tsox_core::fntrace::enter("verify_project_references"); 
        let build_info_file_name = if self.options().suppress_output_path_check != Tristate::True {
            self.opts.config.get_build_info_file_name()
        } else {
            String::new()
        };

        let mut program_diagnostics = std::mem::take(&mut self.program_diagnostics);
        let mut has_emit_blocking = std::mem::take(&mut self.has_emit_blocking_diagnostics);
        let mapper = self.project_reference_file_mapper.clone();
        mapper.range_resolved_project_reference(&mut |_path, config, parent, index| {
            let parent = match parent {
                Some(parent) => parent,
                None => return true,
            };
            let refs = parent.project_references();
            let r = match refs.get(index) {
                Some(r) => r.clone(),
                None => return true,
            };
            let config = match config {
                None => {
                    program_diagnostics.push(create_diagnostic_at_reference_syntax(
                        parent,
                        index,
                        dg::FILE_0_NOT_FOUND,
                        &[r.path.clone()],
                    ));
                    return true;
                }
                Some(config) => config.clone(),
            };
            let ref_options = config.compiler_options();
            if ref_options.composite != Tristate::True || ref_options.no_emit == Tristate::True {
                if !parent.file_names().is_empty() {
                    if ref_options.composite != Tristate::True {
                        program_diagnostics.push(create_diagnostic_at_reference_syntax(
                            &config, index, dg::REFERENCED_PROJECT_0_MUST_HAVE_SETTING_COMPOSITE_COLON_TRUE,
                            &[r.path.clone()],
                        ));
                    }
                    if ref_options.no_emit == Tristate::True {
                        program_diagnostics.push(create_diagnostic_at_reference_syntax(
                            &config, index, dg::REFERENCED_PROJECT_0_MAY_NOT_DISABLE_EMIT,
                            &[r.path.clone()],
                        ));
                    }
                }
            }
            if !build_info_file_name.is_empty()
                && build_info_file_name == config.get_build_info_file_name()
            {
                program_diagnostics.push(create_diagnostic_at_reference_syntax(
                    &config,
                    index,
                    dg::CANNOT_WRITE_FILE_0_BECAUSE_IT_WILL_OVERWRITE_TSBUILDINFO_FILE_GENERATED_BY_REFERENCED_PROJECT_1,
                    &[build_info_file_name.clone(), r.path.clone()],
                ));
                has_emit_blocking.insert(self.to_path(&build_info_file_name).0);
            }
            true
        });
        self.program_diagnostics = program_diagnostics;
        self.has_emit_blocking_diagnostics = has_emit_blocking;
    }

    pub fn verify_compiler_options(&mut self) { ::tsox_core::fntrace::enter("verify_compiler_options"); 
        let mut ctx = OptionDiagCtx::new(self);
        let options = ctx.program.options().clone();

        if !options.base_url.is_empty() {
            let mut use_instead = String::new();
            if !ctx.config_file_path.is_empty() {
                let mut relative = tsox_core::tspath::mig::m3i::get_relative_path_from_file(
                    &ctx.config_file_path,
                    &options.base_url,
                    &ctx.program.compare_paths_options,
                );
                if !(relative.starts_with("./") || relative.starts_with("../")) {
                    relative = format!("./{}", relative);
                }
                let suggestion = tspath::combine_paths(&relative, &["*"]);
                use_instead = format!(
                    "\"paths\": {{\"*\": [{}]}}",
                    serde_json::to_string(&suggestion).unwrap()
                );
            }
            ctx.create_removed_option_diagnostic("baseUrl", "", &use_instead);
        }

        if !options.out_file.is_empty() {
            ctx.create_removed_option_diagnostic("outFile", "", "");
        }

        if options.target == ScriptTarget::ES5 {
            ctx.create_removed_option_diagnostic("target", "ES5", "");
        }

        if options.module == ModuleKind::AMD {
            ctx.create_removed_option_diagnostic("module", "AMD", "");
        }
        if options.module == ModuleKind::System {
            ctx.create_removed_option_diagnostic("module", "System", "");
        }
        if options.module == ModuleKind::UMD {
            ctx.create_removed_option_diagnostic("module", "UMD", "");
        }

        if options.module_resolution == ModuleResolutionKind::Classic {
            ctx.create_removed_option_diagnostic("moduleResolution", "Classic", "");
        }

        if options.always_strict == Tristate::False {
            ctx.create_removed_option_diagnostic("alwaysStrict", "false", "");
        }

        if options.es_module_interop == Tristate::False {
            ctx.create_removed_option_diagnostic("esModuleInterop", "false", "");
        }

        if options.allow_synthetic_default_imports == Tristate::False {
            ctx.create_removed_option_diagnostic("allowSyntheticDefaultImports", "false", "");
        }

        if options.module_resolution == ModuleResolutionKind::Node10 {
            ctx.create_removed_option_diagnostic("moduleResolution", "node10", "");
        }

        if options.downlevel_iteration != Tristate::Unknown {
            ctx.create_removed_option_diagnostic("downlevelIteration", "", "");
        }

        if options.strict_property_initialization == Tristate::True
            && !options.get_strict_option_value(options.strict_null_checks)
        {
            ctx.create_diagnostic_for_option_name(
                dg::OPTION_0_CANNOT_BE_SPECIFIED_WITHOUT_SPECIFYING_OPTION_1,
                "strictPropertyInitialization",
                "strictNullChecks",
                &[],
            );
        }
        if options.exact_optional_property_types == Tristate::True
            && !options.get_strict_option_value(options.strict_null_checks)
        {
            ctx.create_diagnostic_for_option_name(
                dg::OPTION_0_CANNOT_BE_SPECIFIED_WITHOUT_SPECIFYING_OPTION_1,
                "exactOptionalPropertyTypes",
                "strictNullChecks",
                &[],
            );
        }

        if options.isolated_declarations == Tristate::True {
            if options.get_allow_js() {
                ctx.create_diagnostic_for_option_name(
                    dg::OPTION_0_CANNOT_BE_SPECIFIED_WITH_OPTION_1,
                    "allowJs",
                    "isolatedDeclarations",
                    &[],
                );
            }
            if !options.get_emit_declarations() {
                ctx.create_diagnostic_for_option_name(
                    dg::OPTION_0_CANNOT_BE_SPECIFIED_WITHOUT_SPECIFYING_OPTION_1_OR_OPTION_2,
                    "isolatedDeclarations",
                    "declaration",
                    &["composite".to_string()],
                );
            }
        }

        if options.inline_source_map == Tristate::True {
            if options.source_map == Tristate::True {
                ctx.create_diagnostic_for_option_name(
                    dg::OPTION_0_CANNOT_BE_SPECIFIED_WITH_OPTION_1,
                    "sourceMap",
                    "inlineSourceMap",
                    &[],
                );
            }
            if !options.map_root.is_empty() {
                ctx.create_diagnostic_for_option_name(
                    dg::OPTION_0_CANNOT_BE_SPECIFIED_WITH_OPTION_1,
                    "mapRoot",
                    "inlineSourceMap",
                    &[],
                );
            }
        }

        if options.composite == Tristate::True {
            if options.declaration == Tristate::False {
                ctx.create_diagnostic_for_option_name(
                    dg::COMPOSITE_PROJECTS_MAY_NOT_DISABLE_DECLARATION_EMIT,
                    "declaration",
                    "",
                    &[],
                );
            }
            if options.incremental == Tristate::False {
                ctx.create_diagnostic_for_option_name(
                    dg::COMPOSITE_PROJECTS_MAY_NOT_DISABLE_INCREMENTAL_COMPILATION,
                    "declaration",
                    "",
                    &[],
                );
            }
        }

        if options.ts_build_info_file.is_empty()
            && options.incremental == Tristate::True
            && options.config_file_path.is_empty()
        {
            let diag = ctx.create_compiler_options_diagnostic(
                dg::OPTION_INCREMENTAL_IS_ONLY_VALID_WITH_A_KNOWN_CONFIGURATION_FILE_LIKE_TSCONFIG_JSON_OR_WHEN_TSBUILDINFOFILE_IS_EXPLICITLY_PROVIDED,
                vec![],
            );
            ctx.program.program_diagnostics.push(diag);
        }

        ctx.program.verify_project_references();

        if options.composite == Tristate::True {
            let mut root_paths: Set<Path> = Set::new();
            for file_name in ctx.program.opts.config.file_names() {
                root_paths.add(ctx.program.to_path(file_name));
            }
            let config_file_path = ctx.config_file_path.clone();
            for file in &ctx.program.source_files {
                let file_path = ctx.program.to_path(&file.file_name);
                if ctx.program.source_file_may_be_emitted(file, false)
                    && !root_paths.has(&file_path)
                {
                    let args = vec![file.file_name.clone(), config_file_path.clone()];
                    if let Some(include_processor) = ctx.program.include_processor.as_deref_mut() {
                        include_processor.add_processing_diagnostics(vec![
                            processing_diagnostic_explaining_file_include(
                                file_path,
                                dg::FILE_0_IS_NOT_LISTED_WITHIN_THE_FILE_LIST_OF_PROJECT_1_PROJECTS_MUST_LIST_ALL_FILES_OR_USE_AN_INCLUDE_PATTERN,
                                args,
                            ),
                        ]);
                    }
                }
            }
        }

        let paths_entries: Vec<(String, Option<Vec<String>>)> = options
            .paths
            .as_ref()
            .map(|paths| {
                paths
                    .iter()
                    .map(|(key, value)| (key.clone(), Some(value.clone())))
                    .collect()
            })
            .unwrap_or_default();
        for (key, value) in paths_entries {
            if !has_zero_or_one_asterisk_character(&key) {
                ctx.create_diagnostic_for_option_paths(
                    true,
                    &key,
                    dg::PATTERN_0_CAN_HAVE_AT_MOST_ONE_ASTERISK_CHARACTER,
                    &[key.clone()],
                );
            }
            match value.as_ref() {
                None => {
                    ctx.create_diagnostic_for_option_paths(
                        false,
                        &key,
                        dg::SUBSTITUTIONS_FOR_PATTERN_0_SHOULD_BE_AN_ARRAY,
                        &[key.clone()],
                    );
                }
                Some(value) if value.is_empty() => {
                    ctx.create_diagnostic_for_option_paths(
                        false,
                        &key,
                        dg::SUBSTITUTIONS_FOR_PATTERN_0_SHOULDN_T_BE_AN_EMPTY_ARRAY,
                        &[key.clone()],
                    );
                }
                Some(value) => {
                    for (i, subst) in value.iter().enumerate() {
                        if !has_zero_or_one_asterisk_character(subst) {
                            ctx.create_diagnostic_for_option_path_key_value(
                                &key,
                                i,
                                dg::SUBSTITUTION_0_IN_PATTERN_1_CAN_HAVE_AT_MOST_ONE_ASTERISK_CHARACTER,
                                &[subst.clone(), key.clone()],
                            );
                        }
                        if !tspath::path_is_relative(subst) && !tspath::path_is_absolute(subst) {
                            ctx.create_diagnostic_for_option_path_key_value(
                                &key,
                                i,
                                dg::NON_RELATIVE_PATHS_ARE_NOT_ALLOWED_DID_YOU_FORGET_A_LEADING_SLASH,
                                &[],
                            );
                        }
                    }
                }
            }
        }

        if options.source_map != Tristate::True && options.inline_source_map != Tristate::True {
            if options.inline_sources == Tristate::True {
                ctx.create_diagnostic_for_option_name(
                    dg::OPTION_0_CAN_ONLY_BE_USED_WHEN_EITHER_OPTION_INLINESOURCEMAP_OR_OPTION_SOURCEMAP_IS_PROVIDED,
                    "inlineSources",
                    "",
                    &[],
                );
            }
            if !options.source_root.is_empty() {
                ctx.create_diagnostic_for_option_name(
                    dg::OPTION_0_CAN_ONLY_BE_USED_WHEN_EITHER_OPTION_INLINESOURCEMAP_OR_OPTION_SOURCEMAP_IS_PROVIDED,
                    "sourceRoot",
                    "",
                    &[],
                );
            }
        }

        if !options.map_root.is_empty()
            && !(options.source_map == Tristate::True || options.declaration_map == Tristate::True)
        {
            ctx.create_diagnostic_for_option_name(
                dg::OPTION_0_CANNOT_BE_SPECIFIED_WITHOUT_SPECIFYING_OPTION_1_OR_OPTION_2,
                "mapRoot",
                "sourceMap",
                &["declarationMap".to_string()],
            );
        }

        if !options.declaration_dir.is_empty() && !options.get_emit_declarations() {
            ctx.create_diagnostic_for_option_name(
                dg::OPTION_0_CANNOT_BE_SPECIFIED_WITHOUT_SPECIFYING_OPTION_1_OR_OPTION_2,
                "declarationDir",
                "declaration",
                &["composite".to_string()],
            );
        }

        if options.declaration_map == Tristate::True && !options.get_emit_declarations() {
            ctx.create_diagnostic_for_option_name(
                dg::OPTION_0_CANNOT_BE_SPECIFIED_WITHOUT_SPECIFYING_OPTION_1_OR_OPTION_2,
                "declarationMap",
                "declaration",
                &["composite".to_string()],
            );
        }

        if !options.lib.is_empty() && options.no_lib == Tristate::True {
            ctx.create_diagnostic_for_option_name(
                dg::OPTION_0_CANNOT_BE_SPECIFIED_WITH_OPTION_1,
                "lib",
                "noLib",
                &[],
            );
        }

        if options.isolated_modules == Tristate::True || options.verbatim_module_syntax == Tristate::True {
            if options.preserve_const_enums == Tristate::False {
                let option = if options.verbatim_module_syntax == Tristate::True {
                    "verbatimModuleSyntax"
                } else {
                    "isolatedModules"
                };
                ctx.create_diagnostic_for_option_name(
                    dg::OPTION_PRESERVECONSTENUMS_CANNOT_BE_DISABLED_WHEN_0_IS_ENABLED,
                    option,
                    "preserveConstEnums",
                    &[],
                );
            }
        }

        if !options.out_dir.is_empty()
            || !options.root_dir.is_empty()
            || !options.source_root.is_empty()
            || !options.map_root.is_empty()
            || (options.get_emit_declarations() && !options.declaration_dir.is_empty())
        {
            let dir = ctx.program.common_source_directory();
            if !options.out_dir.is_empty()
                && dir.is_empty()
                && ctx
                    .program
                    .source_files
                    .iter()
                    .any(|f| tspath::get_root_length(&f.file_name) > 1)
            {
                ctx.create_diagnostic_for_option_name(
                    dg::CANNOT_FIND_THE_COMMON_SUBDIRECTORY_PATH_FOR_THE_INPUT_FILES,
                    "outDir",
                    "",
                    &[],
                );
            }
        }

        if options.no_emit != Tristate::True
            && options.composite != Tristate::True
            && options.root_dir.is_empty()
            && !options.config_file_path.is_empty()
            && (!options.out_dir.is_empty()
                || (options.get_emit_declarations() && !options.declaration_dir.is_empty())
                || !options.out_file.is_empty())
        {
            let dir = ctx.program.common_source_directory();
            let mut emitted_files = Vec::new();
            for file in &ctx.program.source_files {
                if !file.is_declaration_file
                    && ctx.program.source_file_may_be_emitted(file, false)
                {
                    emitted_files.push(file.file_name.clone());
                }
            }
            let dir59 = get_computed_common_source_directory(
                &emitted_files,
                ctx.program.get_current_directory(),
                ctx.program.use_case_sensitive_file_names(),
            );
            if !dir59.is_empty()
                && tspath::get_canonical_file_name(&dir, ctx.program.use_case_sensitive_file_names())
                    != tspath::get_canonical_file_name(
                        &dir59,
                        ctx.program.use_case_sensitive_file_names(),
                    )
            {
                let option1 = if !options.out_file.is_empty() {
                    "outFile"
                } else if !options.out_dir.is_empty() {
                    "outDir"
                } else {
                    "declarationDir"
                };
                let option2 = if options.out_file.is_empty() && !options.out_dir.is_empty() {
                    "declarationDir"
                } else {
                    ""
                };
                let mut diag = ctx.create_diagnostic_for_option(
                    true,
                    option1,
                    option2,
                    dg::THE_COMMON_SOURCE_DIRECTORY_OF_0_IS_1_THE_ROOTDIR_SETTING_MUST_BE_EXPLICITLY_SET_TO_THIS_OR_ANOTHER_PATH_TO_ADJUST_YOUR_OUTPUT_S_FILE_LAYOUT,
                    &[
                        tspath::get_base_file_name(&options.config_file_path),
                        tsox_core::tspath::mig::m3i::get_relative_path_from_file(
                            &options.config_file_path,
                            &dir59,
                            &ctx.program.compare_paths_options,
                        ),
                    ],
                );
                if let Some(target) = Arc::get_mut(&mut diag) {
                    target.add_message_chain(new_compiler_diagnostic(
                        dg::VISIT_HTTPS_COLON_SLASH_SLASHAKA_MS_SLASHTS6_FOR_MIGRATION_INFORMATION,
                        vec![],
                    ));
                }
                ctx.program.program_diagnostics.push(diag);
            }
        }

        if options.check_js == Tristate::True && !options.get_allow_js() {
            ctx.create_diagnostic_for_option_name(
                dg::OPTION_0_CANNOT_BE_SPECIFIED_WITHOUT_SPECIFYING_OPTION_1,
                "checkJs",
                "allowJs",
                &[],
            );
        }

        if options.emit_declaration_only == Tristate::True && !options.get_emit_declarations() {
            ctx.create_diagnostic_for_option_name(
                dg::OPTION_0_CANNOT_BE_SPECIFIED_WITHOUT_SPECIFYING_OPTION_1_OR_OPTION_2,
                "emitDeclarationOnly",
                "declaration",
                &["composite".to_string()],
            );
        }

        if options.emit_decorator_metadata == Tristate::True
            && options.experimental_decorators != Tristate::True
        {
            ctx.create_diagnostic_for_option_name(
                dg::OPTION_0_CANNOT_BE_SPECIFIED_WITHOUT_SPECIFYING_OPTION_1,
                "emitDecoratorMetadata",
                "experimentalDecorators",
                &[],
            );
        }

        let module_kind = options.get_emit_module_kind();

        if options.allow_importing_ts_extensions == Tristate::True
            && !(options.no_emit == Tristate::True
                || options.emit_declaration_only == Tristate::True
                || options.rewrite_relative_import_extensions == Tristate::True)
        {
            ctx.create_option_value_diagnostic(
                "allowImportingTsExtensions",
                dg::OPTION_ALLOWIMPORTINGTSEXTENSIONS_CAN_ONLY_BE_USED_WHEN_ONE_OF_NOEMIT_EMITDECLARATIONONLY_OR_REWRITERELATIVEIMPORTEXTENSIONS_IS_SET,
                &[],
            );
        }

        let module_resolution = options.get_module_resolution_kind();
        if options.resolve_package_json_exports == Tristate::True
            && !module_resolution_supports_package_json_exports_and_imports(module_resolution)
        {
            ctx.create_diagnostic_for_option_name(
                dg::OPTION_0_CAN_ONLY_BE_USED_WHEN_MODULERESOLUTION_IS_SET_TO_NODE16_NODENEXT_OR_BUNDLER,
                "resolvePackageJsonExports",
                "",
                &[],
            );
        }
        if options.resolve_package_json_imports == Tristate::True
            && !module_resolution_supports_package_json_exports_and_imports(module_resolution)
        {
            ctx.create_diagnostic_for_option_name(
                dg::OPTION_0_CAN_ONLY_BE_USED_WHEN_MODULERESOLUTION_IS_SET_TO_NODE16_NODENEXT_OR_BUNDLER,
                "resolvePackageJsonImports",
                "",
                &[],
            );
        }
        if !options.custom_conditions.is_empty()
            && !module_resolution_supports_package_json_exports_and_imports(module_resolution)
        {
            ctx.create_diagnostic_for_option_name(
                dg::OPTION_0_CAN_ONLY_BE_USED_WHEN_MODULERESOLUTION_IS_SET_TO_NODE16_NODENEXT_OR_BUNDLER,
                "customConditions",
                "",
                &[],
            );
        }

        if module_resolution == ModuleResolutionKind::Bundler
            && !emit_module_kind_is_non_node_esm(module_kind)
            && module_kind != ModuleKind::Preserve
            && module_kind != ModuleKind::CommonJS
        {
            ctx.create_option_value_diagnostic(
                "moduleResolution",
                dg::OPTION_0_CAN_ONLY_BE_USED_WHEN_MODULE_IS_SET_TO_PRESERVE_COMMONJS_OR_ES2015_OR_LATER,
                &["bundler".to_string()],
            );
        }

        if matches!(
            module_kind,
            ModuleKind::Node16 | ModuleKind::Node18 | ModuleKind::Node20 | ModuleKind::NodeNext
        ) && !matches!(
            module_resolution,
            ModuleResolutionKind::Node16 | ModuleResolutionKind::NodeNext
        )
        {
            let module_kind_name = emit_module_kind_name(module_kind);
            let module_resolution_name = if module_kind == ModuleKind::NodeNext {
                "NodeNext"
            } else {
                "Node16"
            };
            ctx.create_option_value_diagnostic(
                "moduleResolution",
                dg::OPTION_MODULERESOLUTION_MUST_BE_SET_TO_0_OR_LEFT_UNSPECIFIED_WHEN_OPTION_MODULE_IS_SET_TO_1,
                &[module_resolution_name.to_string(), module_kind_name],
            );
        } else if matches!(
            module_resolution,
            ModuleResolutionKind::Node16 | ModuleResolutionKind::NodeNext
        ) && !matches!(
            module_kind,
            ModuleKind::Node16 | ModuleKind::Node18 | ModuleKind::Node20 | ModuleKind::NodeNext
        )
        {
            let module_resolution_name = module_resolution_kind_name(module_resolution);
            ctx.create_option_value_diagnostic(
                "module",
                dg::OPTION_MODULE_MUST_BE_SET_TO_0_WHEN_OPTION_MODULERESOLUTION_IS_SET_TO_1,
                &[module_resolution_name.clone(), module_resolution_name],
            );
        }

        if options.no_emit != Tristate::True && options.suppress_output_path_check != Tristate::True {
            let mut emit_files_seen: Set<String> = Set::new();
            let mut verify_emit_file_path = |ctx: &mut OptionDiagCtx, emit_file_name: &str| {
                if emit_file_name.is_empty() {
                    return;
                }
                let emit_file_path = ctx.program.to_path(emit_file_name);
                if ctx.program.files_by_path().contains_key(&emit_file_path.0) {
                    let mut diag = new_compiler_diagnostic(
                        dg::CANNOT_WRITE_FILE_0_BECAUSE_IT_WOULD_OVERWRITE_INPUT_FILE,
                        vec![emit_file_name.to_string()],
                    );
                    if ctx.config_file_path.is_empty() {
                        diag.add_message_chain(new_compiler_diagnostic(
                            dg::ADDING_A_TSCONFIG_JSON_FILE_WILL_HELP_ORGANIZE_PROJECTS_THAT_CONTAIN_BOTH_TYPESCRIPT_AND_JAVASCRIPT_FILES_LEARN_MORE_AT_HTTPS_COLON_SLASH_SLASHAKA_MS_SLASHTSCONFIG,
                            vec![],
                        ));
                    }
                    ctx.program
                        .block_emitting_of_file(emit_file_name, Arc::new(diag));
                }
                let emit_file_key = if !ctx.program.use_case_sensitive_file_names() {
                    tspath::to_file_name_lower_case(&emit_file_path.0)
                } else {
                    emit_file_path.0.clone()
                };
                if emit_files_seen.has(&emit_file_key) {
                    ctx.program.block_emitting_of_file(
                        emit_file_name,
                        Arc::new(new_compiler_diagnostic(
                            dg::CANNOT_WRITE_FILE_0_BECAUSE_IT_WOULD_BE_OVERWRITTEN_BY_MULTIPLE_INPUT_FILES,
                            vec![emit_file_name.to_string()],
                        )),
                    );
                } else {
                    emit_files_seen.add(emit_file_key);
                }
            };

            let source_files_to_emit =
                ctx.program.get_source_files_to_emit(None, false, false);
            let common_dir = ctx.program.common_source_directory();
            let current_dir = ctx.program.get_current_directory().to_string();
            let case_sensitive = ctx.program.use_case_sensitive_file_names();
            for file in &source_files_to_emit {
                for output_name in emitted_output_file_names(
                    &file.file_name,
                    &options,
                    &common_dir,
                    &current_dir,
                    case_sensitive,
                ) {
                    verify_emit_file_path(&mut ctx, &output_name);
                }
            }
            let build_info_file_name = ctx.program.opts.config.get_build_info_file_name();
            verify_emit_file_path(&mut ctx, &build_info_file_name);
        }
    }
}

fn emit_module_kind_is_non_node_esm(
    module_kind: ModuleKind,
) -> bool { ::tsox_core::fntrace::enter("emit_module_kind_is_non_node_esm"); 
    module_kind >= ModuleKind::ES2015
        && module_kind <= ModuleKind::ESNext
}

fn emit_module_kind_name(module_kind: ModuleKind) -> String { ::tsox_core::fntrace::enter("emit_module_kind_name"); 
    match module_kind {
        ModuleKind::Node18 => "node18".to_string(),
        ModuleKind::Node20 => "node20".to_string(),
        ModuleKind::NodeNext => "nodenext".to_string(),
        _ => "node16".to_string(),
    }
}

fn module_resolution_kind_name(module_resolution: ModuleResolutionKind) -> String { ::tsox_core::fntrace::enter("module_resolution_kind_name"); 
    if module_resolution == ModuleResolutionKind::NodeNext {
        "NodeNext".to_string()
    } else {
        "Node16".to_string()
    }
}

fn create_diagnostic_at_reference_syntax(
    config: &ParsedCommandLine,
    index: usize,
    message: Message,
    args: &[String],
) -> Arc<Diagnostic> { ::tsox_core::fntrace::enter("create_diagnostic_at_reference_syntax"); 
    let fallback = || Arc::new(new_compiler_diagnostic(message, args.to_vec()));
    let Some(config_file) = &config.config_file else {
        return fallback();
    };
    let source_file = &config_file.source_file;
    tsox_tsoptions::mig::m5i2::for_each_ts_config_prop_array(
        Some(source_file.as_ref()),
        "references",
        &mut |property| {
            let tsox_frontend::ast::NodeData::PropertyAssignment(data) = &property.data else {
                return None;
            };
            let initializer = data.initializer.as_ref();
            if !tsox_frontend::ast::node_data_generated::is_array_literal_expression(initializer) {
                return None;
            }
            let NodeData::ArrayLiteralExpression(arr) = &initializer.data else {
                return None;
            };
            let element = arr.elements.nodes.get(index)?;
            Some(create_diagnostic_for_node_in_source_file_or_compiler_diagnostic(
                Some(source_file.as_ref()),
                Some(element.as_ref()),
                message,
                args.to_vec(),
            ))
        },
    )
    .map(Arc::new)
    .unwrap_or_else(fallback)
}

fn processing_diagnostic_explaining_file_include(
    file: Path,
    message: Message,
    args: Vec<String>,
) -> Arc<ProcessingDiagnostic> { ::tsox_core::fntrace::enter("processing_diagnostic_explaining_file_include"); 
    Arc::new(ProcessingDiagnostic {
        kind: ProcessingDiagnosticKind::ExplainingFileInclude,
        data: ProcessingDiagnosticData::IncludeExplainingDiagnostic(Box::new(
            IncludeExplainingDiagnostic {
                file,
                diagnostic_reason: std::ptr::null(),
                diagnostic_reason_opt: None,
                message,
                args,
            },
        )),
    })
}

fn output_extension(file_name: &str) -> &'static str { ::tsox_core::fntrace::enter("output_extension"); 
    if tspath::file_extension_is(file_name, ".json") {
        return ".json";
    }
    if tspath::file_extension_is_one_of(file_name, &[".mts", ".mjs"]) {
        return ".mjs";
    }
    if tspath::file_extension_is_one_of(file_name, &[".cts", ".cjs"]) {
        return ".cjs";
    }
    ".js"
}

fn declaration_extension(file_name: &str) -> &'static str { ::tsox_core::fntrace::enter("declaration_extension"); 
    if tspath::file_extension_is_one_of(file_name, &[".mts", ".mjs"]) {
        return ".d.mts";
    }
    if tspath::file_extension_is_one_of(file_name, &[".cts", ".cjs"]) {
        return ".d.cts";
    }
    ".d.ts"
}

fn emitted_output_file_names(
    file_name: &str,
    options: &tsox_core::core::compiler_options::CompilerOptions,
    common_source_directory: &str,
    current_directory: &str,
    use_case_sensitive_file_names: bool,
) -> Vec<String> { ::tsox_core::fntrace::enter("emitted_output_file_names"); 
    if !options.out_file.is_empty() {
        let out_file = &options.out_file;
        let js_path = if tspath::file_extension_is(out_file, ".js") {
            out_file.clone()
        } else {
            format!("{}.js", tspath::remove_file_extension(out_file))
        };
        let mut names = vec![js_path.clone()];
        if options.source_map.is_true() && !options.inline_source_map.is_true() {
            names.push(format!("{js_path}.map"));
        }
        if options.get_emit_declarations() {
            let dts_path = format!("{}.d.ts", tspath::remove_file_extension(out_file));
            names.push(dts_path.clone());
            if options.get_are_declaration_maps_enabled() {
                names.push(format!("{dts_path}.map"));
            }
        }
        return names;
    }

    let path_in_new_dir = |new_dir: &str| -> String {
        super::m4v::get_source_file_path_in_new_dir(
            file_name,
            new_dir,
            current_directory,
            common_source_directory,
            use_case_sensitive_file_names,
        )
    };

    let output_ext = output_extension(file_name);
    let js_path = if !options.out_dir.is_empty() {
        let without_ext = tspath::remove_file_extension(&path_in_new_dir(&options.out_dir));
        format!("{without_ext}{output_ext}")
    } else {
        format!("{}{output_ext}", tspath::remove_file_extension(file_name))
    };

    let mut names = vec![js_path.clone()];
    if options.source_map.is_true() && !options.inline_source_map.is_true() {
        names.push(format!("{js_path}.map"));
    }

    if options.get_emit_declarations() && !file_name.ends_with(".json") {
        let decl_ext = declaration_extension(file_name);
        let dts_path = if !options.declaration_dir.is_empty() {
            let without_ext =
                tspath::remove_file_extension(&path_in_new_dir(&options.declaration_dir));
            format!("{without_ext}{decl_ext}")
        } else if !options.out_dir.is_empty() {
            format!("{}{decl_ext}", tspath::remove_file_extension(&js_path))
        } else {
            format!("{}{decl_ext}", tspath::remove_file_extension(file_name))
        };
        names.push(dts_path.clone());
        if options.get_are_declaration_maps_enabled() {
            names.push(format!("{dts_path}.map"));
        }
    }
    names
}

fn compute_common_source_directory_of_filenames(
    file_names: &[String],
    current_directory: &str,
    use_case_sensitive_file_names: bool,
) -> String { ::tsox_core::fntrace::enter("compute_common_source_directory_of_filenames"); 
    let mut common_path_components: Option<Vec<String>> = None;
    for source_file in file_names {
        let mut source_path_components = tspath::get_path_components(source_file, current_directory);
        source_path_components.pop();
        match &mut common_path_components {
            None => common_path_components = Some(source_path_components),
            Some(common) => {
                let n = common.len().min(source_path_components.len());
                for i in 0..n {
                    if tspath::get_canonical_file_name(&common[i], use_case_sensitive_file_names)
                        != tspath::get_canonical_file_name(
                            &source_path_components[i],
                            use_case_sensitive_file_names,
                        )
                    {
                        if i == 0 {
                            return String::new();
                        }
                        common.truncate(i);
                        break;
                    }
                }
                if source_path_components.len() < common.len() {
                    common.truncate(source_path_components.len());
                }
            }
        }
    }
    match common_path_components {
        Some(components) if !components.is_empty() => {
            tspath::get_path_from_path_components(&components)
        }
        _ => current_directory.to_string(),
    }
}

fn get_computed_common_source_directory(
    emitted_files: &[String],
    current_directory: &str,
    use_case_sensitive_file_names: bool,
) -> String { ::tsox_core::fntrace::enter("get_computed_common_source_directory"); 
    let common_source_directory = compute_common_source_directory_of_filenames(
        emitted_files,
        current_directory,
        use_case_sensitive_file_names,
    );
    if !common_source_directory.is_empty() {
        return tspath::ensure_trailing_directory_separator(&common_source_directory);
    }
    common_source_directory
}
