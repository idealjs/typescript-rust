#![allow(unused_imports)]

use crate::compiler::Program;
use std::sync::{Arc, Mutex};
use tsox_frontend::ast::Node;
use tsox_frontend::ast::SourceFile;
use tsox_frontend::ast::diagnostic::Diagnostic;
use super::m4w_4::sort_and_deduplicate_diagnostics;
use super::m4u_2::CheckerPool;
use tsox_frontend::ast::mig::m3b_2;

pub use super::m4w_5::ModeAwareCacheKey;

impl Program {
    pub fn get_resolved_module(
        &self,
        file: &Arc<SourceFile>,
        module_reference: &str,
        mode: tsox_core::core::compiler_options_kinds::ResolutionMode,
    ) -> Option<tsox_tsoptions::module::ResolvedModule> { ::tsox_core::fntrace::enter("get_resolved_module"); 
        if let Some(resolutions) = self.resolved_modules.get(m3b_2::path(file).as_str()) {
            let key = ModeAwareCacheKey {
                name: module_reference.to_string(),
                mode,
            };
            if let Some(resolved) = resolutions.get(&key) {
                return Some(resolved.clone());
            }
        }
        None
    }

    pub fn get_resolved_module_from_module_specifier(
        &self,
        file: &Arc<SourceFile>,
        module_specifier: &Arc<Node>,
    ) -> Option<tsox_tsoptions::module::ResolvedModule> { ::tsox_core::fntrace::enter("get_resolved_module_from_module_specifier"); 
        if !tsox_frontend::ast::is_string_literal_like(module_specifier) {
            panic!("moduleSpecifier must be a StringLiteralLike");
        }
        let mode = self.get_mode_for_usage_location(file, module_specifier);
        self.get_resolved_module(file, module_specifier.text(), mode)
    }

    pub fn collect_diagnostics(
        &self,
        source_file: Option<&Arc<SourceFile>>,
        concurrent: bool,
        collect: &mut dyn FnMut(&Arc<SourceFile>) -> Vec<Arc<Diagnostic>>,
    ) -> Vec<Arc<Diagnostic>> { ::tsox_core::fntrace::enter("collect_diagnostics"); 
        let result: Vec<Arc<Diagnostic>> = if let Some(source_file) = source_file {
            collect(source_file)
        } else {
            let diagnostics = self.collect_diagnostics_from_files(&self.source_files, concurrent, collect);
            diagnostics.into_iter().flatten().collect()
        };
        filter_and_sort_diagnostics(result)
    }

    pub fn collect_diagnostics_from_files(
        &self,
        source_files: &[Arc<SourceFile>],
        _concurrent: bool,
        collect: &mut dyn FnMut(&Arc<SourceFile>) -> Vec<Arc<Diagnostic>>,
    ) -> Vec<Vec<Arc<Diagnostic>>> { ::tsox_core::fntrace::enter("collect_diagnostics_from_files"); 
        source_files
            .iter()
            .map(|file| collect(file))
            .collect()
    }

    pub fn collect_checker_diagnostics(
        &self,
        source_file: Option<&Arc<SourceFile>>,
        collect: &mut (dyn FnMut(&tsox_checker::checker::Checker, &Arc<SourceFile>) -> Vec<Arc<Diagnostic>> + Send),
    ) -> Vec<Arc<Diagnostic>> { ::tsox_core::fntrace::enter("collect_checker_diagnostics"); 
        if let Some(source_file) = source_file {
            if self.skip_type_checking(source_file, false) {
                return Vec::new();
            }
            let checker = self.get_type_checker_for_file_exclusive(source_file);
            let result = collect(&checker, source_file);
            drop(checker);
            return filter_and_sort_diagnostics(result);
        }
        let collected = self.collect_checker_diagnostics_from_files(&self.source_files, collect);
        filter_and_sort_diagnostics(collected.into_iter().flatten().collect())
    }

    pub fn collect_checker_diagnostics_from_files(
        &self,
        source_files: &[Arc<SourceFile>],
        collect: &mut (dyn FnMut(&tsox_checker::checker::Checker, &Arc<SourceFile>) -> Vec<Arc<Diagnostic>> + Send),
    ) -> Vec<Vec<Arc<Diagnostic>>> { ::tsox_core::fntrace::enter("collect_checker_diagnostics_from_files"); 
        let mut diagnostics: Vec<Vec<Arc<Diagnostic>>> = vec![Vec::new(); source_files.len()];
        if let Some(compiler_checker_pool) = self.compiler_checker_pool.get() {
            let diagnostics = Mutex::new(diagnostics);
            let mut collect_diagnostic = |c: &mut tsox_checker::checker::Checker,
                                          file_index: usize,
                                          file: &Arc<SourceFile>| {
                diagnostics.lock().unwrap()[file_index] = collect(c, file);
            };
            compiler_checker_pool.for_each_checker_group_do(
                source_files,
                self.single_threaded(),
                &mut collect_diagnostic,
            );
            return diagnostics.into_inner().unwrap();
        }
        for (i, file) in source_files.iter().enumerate() {
            if self.skip_type_checking(file, false) {
                continue;
            }
            let checker = self
                .checker_pool
                .get()
                .expect("checker pool is initialized after program construction")
                .get_checker(Some(file));
            diagnostics[i] = collect(&checker, file);
            drop(checker);
        }
        diagnostics
    }

    pub fn get_syntactic_diagnostics(
        &self,
        source_file: Option<&Arc<SourceFile>>,
    ) -> Vec<Arc<Diagnostic>> { ::tsox_core::fntrace::enter("get_syntactic_diagnostics"); 
        self.collect_diagnostics(source_file, false, &mut |file: &Arc<SourceFile>| {
            let mut diags: Vec<Arc<Diagnostic>> = m3b_2::diagnostics(file)
                .iter()
                .chain(m3b_2::js_diagnostics(file).iter())
                .map(|d| Arc::new(d.clone()))
                .collect();
            if tsox_frontend::ast::is_source_file_js(file)
                && !tsox_frontend::ast::mig::m3f_4::is_check_js_enabled_for_file(file, self.options())
            {
                diags.extend(get_additional_js_syntactic_diagnostics(file, self.options()));
            }
            diags
        })
    }

    pub fn bind_source_files(&self) { ::tsox_core::fntrace::enter("bind_source_files"); 
        for file in &self.source_files {
            tsox_checker::binder::bind_source_file(file);
        }
    }

    pub fn get_bind_diagnostics(
        &self,
        source_file: Option<&Arc<SourceFile>>,
    ) -> Vec<Arc<Diagnostic>> { ::tsox_core::fntrace::enter("get_bind_diagnostics"); 
        if let Some(source_file) = source_file {
            tsox_checker::binder::bind_source_file(source_file);
        } else {
            self.bind_source_files();
        }
        self.collect_diagnostics(source_file, false, &mut |file: &Arc<SourceFile>| {
            file.bind_diagnostics()
        })
    }

    pub fn get_semantic_diagnostics_without_no_emit_filtering(
        &self,
        source_files: &[Arc<SourceFile>],
    ) -> std::collections::HashMap<String, Vec<Arc<Diagnostic>>> { ::tsox_core::fntrace::enter("get_semantic_diagnostics_without_no_emit_filtering"); 
        let all_diags = self.collect_checker_diagnostics_from_files(
            source_files,
            &mut |c, file| self.get_bind_and_check_diagnostics_with_checker(c, file),
        );
        let mut result = std::collections::HashMap::with_capacity(source_files.len());
        for (i, diags) in all_diags.into_iter().enumerate() {
            result.insert(m3b_2::path(&source_files[i]), filter_and_sort_diagnostics(diags));
        }
        result
    }

    pub fn get_suggestion_diagnostics(
        &self,
        source_file: Option<&Arc<SourceFile>>,
    ) -> Vec<Arc<Diagnostic>> { ::tsox_core::fntrace::enter("get_suggestion_diagnostics"); 
        self.collect_checker_diagnostics(
            source_file,
            &mut |c, file| self.get_suggestion_diagnostics_with_checker(c, file),
        )
    }

    pub fn get_program_diagnostics(&self) -> Vec<Arc<Diagnostic>> { ::tsox_core::fntrace::enter("get_program_diagnostics"); 
        let mut all: Vec<Arc<Diagnostic>> = self.program_diagnostics.clone();
        all.extend(self.content_mapper_diagnostics.iter().cloned());
        all.extend(self.content_mapper_option_diagnostics.iter().cloned());
        if let Some(include_processor) = self.include_processor.as_deref() {
            all.extend(
                include_processor
                    .get_diagnostics(self)
                    .get_global_diagnostics()
                    .into_iter()
                    .map(Arc::new),
            );
        }
        sort_and_deduplicate_diagnostics(all)
    }

    pub fn collect_content_mapper_option_diagnostics(&mut self) { ::tsox_core::fntrace::enter("collect_content_mapper_option_diagnostics"); 
        let project = self.content_mapper_project();
        let Some(project) = project else {
            return;
        };
        let option_diagnostics = project.diagnostics();
        let config = self.opts.config.clone();
        self.content_mapper_option_diagnostics = option_diagnostics
            .iter()
            .filter_map(|diagnostic| {
                let mapper = tsox_tsoptions::mig::m5h_3::ContentMapper {
                    definition: tsox_tsoptions::mig::m5h_3::ContentMapperDefinition {
                        package: diagnostic.mapper.definition.package.clone(),
                        extensions: diagnostic.mapper.definition.extensions.clone(),
                        options: diagnostic.mapper.definition.options.clone(),
                    },
                    ..Default::default()
                };
                let config_file = config.config_file.as_ref()?;
                let path: Vec<tsox_tsoptions::mig::m5i2_3::OptionPathSegment> = diagnostic
                    .path
                    .iter()
                    .map(|segment| tsox_tsoptions::mig::m5i2_3::OptionPathSegment {
                        property: segment.property.clone(),
                        index: usize::try_from(segment.index).unwrap_or(0),
                        is_index: segment.is_index,
                    })
                    .collect();
                let Some((_, loc)) =
                    tsox_tsoptions::mig::m5i2_3::get_content_mapper_option_diagnostic_location(
                        Some(&config),
                        &mapper,
                        &path,
                    )
                else {
                    return None;
                };
                Some(Arc::new(
                    tsox_frontend::ast::mig::m3d_2::new_external_diagnostic(
                        Some(config_file.source_file.clone()),
                        loc,
                        diagnostic.source.clone(),
                        tsox_core::diagnostics::Category::Error,
                        diagnostic.code,
                        diagnostic.message_text.clone(),
                    ),
                ))
            })
            .collect();
    }

    pub fn get_include_processor_diagnostics(
        &self,
        source_file: &Arc<SourceFile>,
    ) -> Vec<Arc<Diagnostic>> { ::tsox_core::fntrace::enter("get_include_processor_diagnostics"); 
        if self.skip_type_checking(source_file, false) {
            return Vec::new();
        }
        let diagnostics = match self.include_processor.as_deref() {
            Some(include_processor) => {
                let mut diags: Vec<Arc<Diagnostic>> = include_processor
                    .get_diagnostics(self)
                    .get_for_file(&source_file.file_name)
                    .into_iter()
                    .map(Arc::new)
                    .collect();
                diags.sort_by(|a, b| {
                    tsox_frontend::ast::mig::m3d_2::compare_diagnostics(a, b)
                });
                diags
            }
            None => Vec::new(),
        };
        let (filtered, _) =
            self.get_diagnostics_with_preceding_directives(source_file, diagnostics);
        filtered
    }

    pub fn skip_type_checking(&self, source_file: &Arc<SourceFile>, ignore_no_check: bool) -> bool { ::tsox_core::fntrace::enter("skip_type_checking"); 
        (!ignore_no_check && self.options().no_check.is_true())
            || self.options().skip_lib_check.is_true() && source_file.is_declaration_file
            || self.options().skip_default_lib_check.is_true()
                && self.is_source_file_default_library(m3b_2::path(source_file).as_str())
            || self.is_source_from_project_reference(m3b_2::path(source_file).as_str())
            || !self.can_include_bind_and_check_diagnostics(source_file)
    }
}

pub fn get_additional_js_syntactic_diagnostics(
    file: &Arc<SourceFile>,
    options: &tsox_core::core::compiler_options::CompilerOptions,
) -> Vec<Arc<Diagnostic>> { ::tsox_core::fntrace::enter("get_additional_js_syntactic_diagnostics"); 
    if options.experimental_decorators.is_true() {
        return Vec::new();
    }
    let mut diags: Vec<Arc<Diagnostic>> = Vec::new();
    use tsox_frontend::ast::node_data_generated::for_each_child;
    fn walk(
        node: &tsox_frontend::ast::Node,
        file: &Arc<SourceFile>,
        diags: &mut Vec<Arc<Diagnostic>>,
    ) { ::tsox_core::fntrace::enter("walk"); 
        if node.subtree_facts()
            & tsox_frontend::ast::subtree_facts::SubtreeFacts::CONTAINS_DECORATORS
            == tsox_frontend::ast::subtree_facts::SubtreeFacts::empty()
        {
            return;
        }
        if node.kind == tsox_frontend::ast::SyntaxKind::Parameter
            && tsox_frontend::ast::has_syntactic_modifier(
                node,
                tsox_frontend::ast::ModifierFlags::Decorator,
            )
        {
            let decorator = node
                .modifier_nodes()
                .iter()
                .find(|m| tsox_frontend::ast::is_decorator(m));
            if let Some(decorator) = decorator {
                diags.push(Arc::new(Diagnostic::new(
                    Some(file.clone()),
                    decorator.loc,
                    tsox_core::diagnostics::messages_generated::DECORATORS_ARE_NOT_VALID_HERE,
                    Vec::new(),
                )));
            }
        }
        for_each_child(node, |child| {
            walk(child, file, diags);
            false
        });
    }
    for_each_child(&file.node, |child| {
        walk(child, file, &mut diags);
        false
    });
    diags
}

pub fn filter_and_sort_diagnostics(diags: Vec<Arc<Diagnostic>>) -> Vec<Arc<Diagnostic>> { ::tsox_core::fntrace::enter("filter_and_sort_diagnostics"); 
    sort_and_deduplicate_diagnostics(diags)
}
