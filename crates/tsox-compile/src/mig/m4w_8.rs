#![allow(unused_imports)]

use super::m4w_4::ProgramLike;
use super::m4w_5::ModeAwareCache;
use crate::compiler::Program;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tsox_frontend::ast::SourceFile;
use tsox_frontend::ast::diagnostic::Diagnostic;
use tsox_core::tspath;

pub fn handle_no_emit_options(
    program: &dyn ProgramLike,
    files: Option<&[Arc<SourceFile>]>,
    emit_build_info: Option<&dyn Fn() -> Option<tsox_emit::emitter::EmitResult>>,
) -> Option<tsox_emit::emitter::EmitResult> { ::tsox_core::fntrace::enter("handle_no_emit_options"); 
    if !program.options().no_emit.is_true() {
        if !program.options().no_emit_on_error.is_true() {
            return None;
        }

        let diagnostics = get_diagnostics_of_any_program(
            program,
            files,
            true,
            &mut |ctx_file| program.get_bind_diagnostics(ctx_file),
            &mut |ctx_file| program.get_semantic_diagnostics(ctx_file),
        );
        if diagnostics.is_empty() {
            return None;
        }
        return Some(tsox_emit::emitter::EmitResult {
            diagnostics: diagnostics
                .iter()
                .map(|diagnostic| {
                    diagnostic.localize(tsox_core::locale::Locale::default())
                })
                .collect(),
            emit_skipped: true,
            ..Default::default()
        });
    }
    if files.is_some() {
        return Some(tsox_emit::emitter::EmitResult {
            emit_skipped: true,
            ..Default::default()
        });
    }
    if let Some(emit_build_info) = emit_build_info {
        let result = emit_build_info();
        if let Some(result) = result {
            return Some(result);
        }
    }
    Some(tsox_emit::emitter::EmitResult::default())
}

pub fn get_diagnostics_of_any_program(
    program: &dyn ProgramLike,
    files: Option<&[Arc<SourceFile>]>,
    skip_no_emit_check_for_dts_diagnostics: bool,
    get_bind_diagnostics: &mut dyn FnMut(Option<&Arc<SourceFile>>) -> Vec<Arc<Diagnostic>>,
    get_semantic_diagnostics: &mut dyn FnMut(Option<&Arc<SourceFile>>) -> Vec<Arc<Diagnostic>>,
) -> Vec<Arc<Diagnostic>> { ::tsox_core::fntrace::enter("get_diagnostics_of_any_program"); 
    let mut all_diagnostics = program.get_config_file_parsing_diagnostics();
    let config_file_parsing_diagnostics_length = all_diagnostics.len();

    let mut append_diagnostics_for_all_files =
        |diagnostics: Vec<Arc<Diagnostic>>,
         get_diagnostics: &mut dyn FnMut(Option<&Arc<SourceFile>>) -> Vec<Arc<Diagnostic>>|
         -> Vec<Arc<Diagnostic>> {
            let mut diagnostics = diagnostics;
            match files {
                None => {
                    diagnostics.extend(get_diagnostics(None));
                    diagnostics
                }
                Some(files) => {
                    for file in files {
                        diagnostics.extend(get_diagnostics(Some(file)));
                    }
                    diagnostics
                }
            }
        };

    let syntactic_diagnostics = append_diagnostics_for_all_files(Vec::new(), &mut |file| {
        program.get_syntactic_diagnostics(file)
    });
    if !syntactic_diagnostics.is_empty() {
        all_diagnostics.extend(program.program().content_mapper_diagnostics.iter().cloned());
    }
    all_diagnostics.extend(syntactic_diagnostics);

    if all_diagnostics.len() == config_file_parsing_diagnostics_length {
        all_diagnostics.extend(program.get_program_diagnostics());

        append_diagnostics_for_all_files(Vec::new(), get_bind_diagnostics);

        if program.options().list_files_only.is_false_or_unknown() {
            all_diagnostics.extend(program.get_global_diagnostics());

            if all_diagnostics.len() == config_file_parsing_diagnostics_length {
                all_diagnostics = append_diagnostics_for_all_files(
                    all_diagnostics,
                    get_semantic_diagnostics,
                );
                all_diagnostics.extend(program.get_global_diagnostics());
            }

            if (skip_no_emit_check_for_dts_diagnostics || program.options().no_emit.is_true())
                && program.options().get_emit_declarations()
                && all_diagnostics.len() == config_file_parsing_diagnostics_length
            {
                all_diagnostics = append_diagnostics_for_all_files(all_diagnostics, &mut |file| {
                    program.get_declaration_diagnostics(file)
                });
            }
        }
    }
    all_diagnostics
}

pub fn emit_module_kind_is_non_node_esm(
    module_kind: tsox_core::core::compiler_options::ModuleKind,
) -> bool { ::tsox_core::fntrace::enter("emit_module_kind_is_non_node_esm"); 
    module_kind >= tsox_core::core::compiler_options::ModuleKind::ES2015
        && module_kind <= tsox_core::core::compiler_options::ModuleKind::ESNext
}

fn maps_equal_by<K: std::hash::Hash + Eq, V, F: Fn(&V, &V) -> bool>(
    a: &HashMap<K, V>,
    b: &HashMap<K, V>,
    eq: F,
) -> bool { ::tsox_core::fntrace::enter("maps_equal_by"); 
    a.len() == b.len()
        && a.iter().all(|(k, va)| match b.get(k) {
            Some(vb) => eq(va, vb),
            None => false,
        })
}

impl Program {
    pub fn get_symlink_cache(&self) -> &tsox_core::symlinks::KnownSymlinks { ::tsox_core::fntrace::enter("get_symlink_cache"); 
        self.known_symlinks.get_value(|| {
            let known_symlinks = tsox_core::symlinks::KnownSymlinks::new(
                self.get_current_directory(),
                self.use_case_sensitive_file_names(),
            );

            if !self.resolved_modules.is_empty() || !self.type_resolutions_in_file.is_empty() {
                known_symlinks.set_symlinks_from_resolutions(
                    |cb: &dyn Fn(&str, &str)| {
                        self.for_each_resolved_module(
                            &mut |resolution, _name, _mode, _file_path| {
                                cb(
                                    &resolution.original_path,
                                    &resolution.resolved_file_name,
                                )
                            },
                            None,
                        );
                    },
                    |cb: &dyn Fn(&str, &str)| {
                        self.for_each_resolved_type_reference_directive(
                            &mut |resolution, _name, _mode, _file_path| {
                                cb(
                                    &resolution.original_path,
                                    &resolution.resolved_file_name,
                                )
                            },
                            None,
                        );
                    },
                );
            }

            let mut seen_package_jsons: HashSet<tsox_core::tspath::directory_separator::Path> =
                HashSet::new();
            for (file_path, meta) in &self.source_file_meta_datas {
                if meta.package_json_directory.is_empty() {
                    continue;
                }
                let Some(file) = self.get_source_file_by_path(file_path) else {
                    continue;
                };
                if !self.source_file_may_be_emitted(&file, false) {
                    continue;
                }
                if !seen_package_jsons
                    .insert(self.to_path(&meta.package_json_directory))
                {
                    continue;
                }
                let package_json_name =
                    tspath::combine_paths(&meta.package_json_directory, &["package.json"]);
                let Some(info) = self.get_package_json_info(&package_json_name) else {
                    continue;
                };
                let Some(contents) = info.get_contents() else {
                    continue;
                };

                for dep in contents.dependency_fields.get_runtime_dependency_names() {
                    let possible_directory_path = self.to_path(&tspath::combine_paths(
                        &meta.package_json_directory,
                        &[&format!("node_modules/{dep}")],
                    ));
                    if known_symlinks.has_directory(&possible_directory_path) {
                        continue;
                    }
                    if !dep.starts_with("@types") {
                        let possible_types_directory_path = self.to_path(&tspath::combine_paths(
                            &meta.package_json_directory,
                            &[&format!(
                                "node_modules/{}",
                                tsox_tsoptions::module::get_types_package_name(&dep)
                            )],
                        ));
                        if known_symlinks.has_directory(&possible_types_directory_path) {
                            continue;
                        }
                    }

                    if let Some(package_resolution) = super::m4w::resolve_package_directory(
                        &self.resolver,
                        &dep,
                        &package_json_name,
                    ) {
                        if package_resolution.is_resolved()
                            && !package_resolution.original_path.is_empty()
                        {
                            known_symlinks.process_resolution(
                                &tspath::combine_paths(
                                    &package_resolution.original_path,
                                    &["package.json"],
                                ),
                                &tspath::combine_paths(
                                    &package_resolution.resolved_file_name,
                                    &["package.json"],
                                ),
                            );
                        }
                    }
                }
            }
            known_symlinks
        })
    }

    pub fn resolve_module_name(
        &self,
        module_name: &str,
        containing_file: &str,
        resolution_mode: tsox_core::core::compiler_options_kinds::ResolutionMode,
    ) -> Option<tsox_tsoptions::module::ResolvedModule> { ::tsox_core::fntrace::enter("resolve_module_name"); 
        let (resolved, _) = self
            .resolver
            .resolve_module_name(module_name, containing_file, resolution_mode, None);
        resolved
    }

    pub fn for_each_resolved_module(
        &self,
        callback: &mut dyn FnMut(
            &tsox_tsoptions::module::ResolvedModule,
            &str,
            tsox_core::core::compiler_options_kinds::ResolutionMode,
            &str,
        ),
        file: Option<&Arc<SourceFile>>,
    ) { ::tsox_core::fntrace::enter("for_each_resolved_module"); 
        for_each_resolution(&self.resolved_modules, callback, file);
    }

    pub fn for_each_resolved_type_reference_directive(
        &self,
        callback: &mut dyn FnMut(
            &tsox_tsoptions::module::ResolvedTypeReferenceDirective,
            &str,
            tsox_core::core::compiler_options_kinds::ResolutionMode,
            &str,
        ),
        file: Option<&Arc<SourceFile>>,
    ) { ::tsox_core::fntrace::enter("for_each_resolved_type_reference_directive"); 
        for_each_resolution(&self.type_resolutions_in_file, callback, file);
    }
}

pub fn for_each_resolution<T>(
    resolution_cache: &HashMap<String, ModeAwareCache<T>>,
    callback: &mut dyn FnMut(&T, &str, tsox_core::core::compiler_options_kinds::ResolutionMode, &str),
    file: Option<&Arc<SourceFile>>,
) { ::tsox_core::fntrace::enter("for_each_resolution"); 
    if let Some(file) = file {
        if let Some(resolutions) = resolution_cache.get(file.file_name.as_str()) {
            for (key, resolution) in resolutions {
                callback(resolution, &key.name, key.mode, file.file_name.as_str());
            }
        }
    } else {
        for (file_path, resolutions) in resolution_cache {
            for (key, resolution) in resolutions {
                callback(resolution, &key.name, key.mode, file_path.as_str());
            }
        }
    }
}
