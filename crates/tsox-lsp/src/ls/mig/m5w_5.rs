#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashSet;
use std::sync::Arc;

use tsox_core::core::compiler_options::{CompilerOptions, ResolutionMode};
use tsox_core::core::text::TextPos;
use tsox_frontend::ast::{self, Node, NodeData, SourceFile, Symbol};

use super::m5w_6::{
    find_closest_declaration_node, find_containing_module_specifier,
    get_candidate_source_declaration_names, get_file_and_start_pos_from_declaration,
    get_reference_at_position, get_source_def_checker_info, get_source_definition_entry_declarations,
    has_concrete_source_declarations, is_default_import_name, filter_preferred_source_declarations,
    unique_declaration_nodes,
};
use crate::ls::mig::m5s::SpanFeature;

struct ProgramFsHost {
    fs: Arc<dyn tsox_tsoptions::vfs::FS>,
    current_directory: String,
}

impl tsox_tsoptions::module::ResolutionHost for ProgramFsHost {
    fn fs(&self) -> &dyn tsox_tsoptions::vfs::FS { ::tsox_core::fntrace::enter("fs"); 
        self.fs.as_ref()
    }
    fn get_current_directory(&self) -> &str { ::tsox_core::fntrace::enter("get_current_directory"); 
        &self.current_directory
    }
}

pub struct SourceDefResolver {
    pub ls: *const crate::ls::language_service::LanguageService,
    pub fs: Arc<dyn tsox_tsoptions::vfs::FS>,
    pub options: CompilerOptions,
    pub get_source_file: Box<dyn Fn(&str) -> Option<Arc<SourceFile>>>,
    pub resolve_from: String,
    pub resolver_cell: std::cell::RefCell<tsox_tsoptions::module::Resolver>,
    pub parsed_files:
        std::cell::RefCell<std::collections::HashMap<String, Option<Arc<SourceFile>>>>,
}

impl SourceDefResolver {
    fn ls(&self) -> &crate::ls::language_service::LanguageService { ::tsox_core::fntrace::enter("ls"); 
        unsafe { &*self.ls }
    }
}

pub fn new_source_def_resolver(
    l: &crate::ls::language_service::LanguageService,
    program: &Arc<tsox_compile::compiler::Program>,
    resolve_from: &str,
) -> SourceDefResolver { ::tsox_core::fntrace::enter("new_source_def_resolver"); 
    let options = program.options().clone();
    let mut no_dts_options = options.clone();
    no_dts_options.no_dts_resolution = tsox_core::core::tristate::Tristate::True;
    let fs = program.host().fs_arc();
    let host = ProgramFsHost {
        fs: std::sync::Arc::clone(&fs),
        current_directory: program.host().current_directory().to_string(),
    };
    SourceDefResolver {
        ls: l as *const _,
        fs,
        options,
        get_source_file: {
            let program = Arc::clone(program);
            Box::new(move |file_name: &str| program.get_source_file(file_name))
        },
        resolve_from: resolve_from.to_string(),
        resolver_cell: std::cell::RefCell::new(tsox_tsoptions::module::Resolver::new(
            std::sync::Arc::new(host),
            std::sync::Arc::new(no_dts_options),
            String::new(),
            String::new(),
        )),
        parsed_files: std::cell::RefCell::new(std::collections::HashMap::new()),
    }
}

impl crate::ls::language_service::LanguageService {
    pub fn provide_source_definition_at_position(
        &self,
        program: &Arc<tsox_compile::compiler::Program>,
        file: &Arc<SourceFile>,
        text_pos: TextPos,
        client_supports_link: bool,
    ) -> crate::ls::mig::m5s::DefinitionResponse { ::tsox_core::fntrace::enter("provide_source_definition_at_position"); 
        let pos = text_pos as usize;
        let resolver = new_source_def_resolver(self, program, &file.file_name);
        let node = match tsox_frontend::astnav::get_touching_property_name(&file.node, pos) {
            Some(node) => node,
            None => return crate::ls::mig::m5s::DefinitionResponse::default(),
        };

        if node.kind == tsox_frontend::ast::SyntaxKind::SourceFile {
            if let Some((declarations, reference)) =
                resolver.resolve_triple_slash_reference(file, pos, program)
            {
                let origin_selection_range = self.create_lsp_range_from_bounds(
                    reference.range.pos as usize,
                    reference.range.end as usize,
                    &m5w_script_view_for_file(file),
                );
                return self.create_definition_locations(
                    origin_selection_range,
                    client_supports_link,
                    declarations,
                    None,
                    SpanFeature::Definition,
                );
            }
            return crate::ls::mig::m5s::DefinitionResponse::default();
        }

        let origin_selection_range = self.m5w_create_lsp_range_from_node(&node, file);

        let containing_module_specifier = find_containing_module_specifier(&node);
        if let Some(specifier) = &containing_module_specifier {
            if Arc::ptr_eq(&node, specifier) {
                let specifier_mode = program.get_mode_for_usage_location(file, specifier);
                let specifier_text = ast::node_text(specifier);
                if !resolver.resolve_implementation(&specifier_text, specifier_mode).is_empty() {
                    if let Some(source_file) =
                        resolver.get_or_parse_source_file(&specifier_text)
                    {
                        return self.create_definition_locations(
                            origin_selection_range,
                            client_supports_link,
                            get_source_definition_entry_declarations(&source_file),
                            None,
                            SpanFeature::Definition,
                        );
                    }
                }
                return self.provide_definition_at_position(
                    program,
                    file,
                    text_pos as u32,
                    client_supports_link,
                );
            }
        }

        let mut resolved_impl_file = String::new();
        if let Some(specifier) = &containing_module_specifier {
            let specifier_mode = program.get_mode_for_usage_location(file, specifier);
            resolved_impl_file =
                resolver.resolve_implementation(&ast::node_text(specifier), specifier_mode);
        }

        if !resolved_impl_file.is_empty() {
            let names = get_candidate_source_declaration_names(Some(&node), None);
            let module_results =
                resolver.search_implementation_file(&node, &resolved_impl_file, &names);
            if !module_results.is_empty() {
                if (!tsox_frontend::ast::mig::m3g_3::is_part_of_type_node(&node)
                    && !tsox_frontend::ast::mig::m3g_3::is_part_of_type_only_import_or_export_declaration(
                        &node,
                    ))
                    || has_concrete_source_declarations(&module_results)
                {
                    return self.create_definition_locations(
                        origin_selection_range,
                        client_supports_link,
                        unique_declaration_nodes(&module_results),
                        None,
                        SpanFeature::Definition,
                    );
                }
            }
        }

        let (checker_declarations, module_specifier) =
            get_source_def_checker_info(program, file, &node);

        let declarations = resolver.resolve_from_checker_info(
            &node,
            &resolved_impl_file,
            &checker_declarations,
            &module_specifier,
        );
        if declarations.is_empty() {
            if !containing_module_specifier.is_none()
                && !resolved_impl_file.is_empty()
                && !has_concrete_source_declarations(&checker_declarations)
            {
                if let Some(source_file) = resolver.get_or_parse_source_file(&resolved_impl_file) {
                    return self.create_definition_locations(
                        origin_selection_range,
                        client_supports_link,
                        get_source_definition_entry_declarations(&source_file),
                        None,
                        SpanFeature::Definition,
                    );
                }
            }
            return self.provide_definition_at_position(
                program,
                file,
                text_pos as u32,
                client_supports_link,
            );
        }
        self.create_definition_locations(
            origin_selection_range,
            client_supports_link,
            declarations,
            None,
            SpanFeature::Definition,
        )
    }

    pub fn m5w_create_lsp_range_from_node(
        &self,
        node: &Arc<Node>,
        file: &Arc<SourceFile>,
    ) -> crate::lsp::lsproto_lsp::Range { ::tsox_core::fntrace::enter("m5w_create_lsp_range_from_node"); 
        self.create_lsp_range_from_bounds(
            node.pos(),
            node.end(),
            &m5w_script_view_for_file(file),
        )
    }
}

pub fn m5w_script_view_for_file(file: &Arc<SourceFile>) -> crate::mig::m5u_conv::SourceFileScriptView { ::tsox_core::fntrace::enter("m5w_script_view_for_file"); 
    crate::mig::m5u_conv::SourceFileScriptView {
        file: Arc::clone(file),
    }
}

impl SourceDefResolver {
    pub fn resolve_from_checker_info(
        &self,
        node: &Arc<Node>,
        resolved_impl_file: &str,
        checker_declarations: &[Arc<Node>],
        module_specifier: &str,
    ) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("resolve_from_checker_info"); 
        let mut resolved_impl_file = resolved_impl_file.to_string();
        if resolved_impl_file.is_empty() && !module_specifier.is_empty() {
            resolved_impl_file =
                self.resolve_implementation(module_specifier, self.infer_implied_node_format(&self.resolve_from));
        }

        if checker_declarations.is_empty() && !resolved_impl_file.is_empty() {
            let names = get_candidate_source_declaration_names(Some(node), None);
            let results = self.search_implementation_file(node, &resolved_impl_file, &names);
            if !results.is_empty() {
                return unique_declaration_nodes(&results);
            }
        }

        let mut declarations: Vec<Arc<Node>> = Vec::new();
        for declaration in checker_declarations {
            declarations.extend(self.map_declaration_to_source(node, declaration, &resolved_impl_file));
        }
        let declarations = unique_declaration_nodes(&declarations);
        if has_concrete_source_declarations(&declarations) {
            return declarations;
        }
        Vec::new()
    }

    pub fn resolve_triple_slash_reference(
        &self,
        file: &Arc<SourceFile>,
        pos: usize,
        program: &Arc<tsox_compile::compiler::Program>,
    ) -> Option<(Vec<Arc<Node>>, Arc<tsox_frontend::ast::node_source_file::FileReference>)> { ::tsox_core::fntrace::enter("resolve_triple_slash_reference"); 
        let reference = get_reference_at_position(file, pos, program)?;
        let reference_file = reference.file.as_ref()?;
        let reference_node = reference.reference.clone()?;

        if !reference_file.is_declaration_file {
            return Some((
                get_source_definition_entry_declarations(reference_file),
                reference_node,
            ));
        }

        let dts_file_name = reference_file.file_name.clone();
        let preferred_mode = self.infer_implied_node_format(&dts_file_name);
        let implementation_file =
            self.find_implementation_file_from_dts_file_name(&dts_file_name, preferred_mode);
        if implementation_file.is_empty() {
            return None;
        }
        let source_file = self.get_or_parse_source_file(&implementation_file)?;
        Some((
            get_source_definition_entry_declarations(&source_file),
            reference_node,
        ))
    }

    pub fn search_implementation_file(
        &self,
        original_node: &Arc<Node>,
        implementation_file: &str,
        names: &[String],
    ) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("search_implementation_file"); 
        if implementation_file.is_empty() {
            return Vec::new();
        }
        let Some(source_file) = self.get_or_parse_source_file(implementation_file) else {
            return Vec::new();
        };
        if is_default_import_name(original_node) {
            let mut seen = HashSet::new();
            seen.insert(implementation_file.to_string());
            let default_declarations =
                self.find_declarations_in_file(implementation_file, &["default".to_string()], &mut seen);
            if !default_declarations.is_empty() {
                return filter_preferred_source_declarations(original_node, &default_declarations);
            }
            return get_source_definition_entry_declarations(&source_file);
        }
        let mut seen = HashSet::new();
        seen.insert(implementation_file.to_string());
        let declarations = self.find_declarations_in_file(implementation_file, names, &mut seen);
        if !declarations.is_empty() {
            return filter_preferred_source_declarations(original_node, &declarations);
        }
        Vec::new()
    }

    pub fn map_declaration_to_source(
        &self,
        original_node: &Arc<Node>,
        declaration: &Arc<Node>,
        resolved_impl_file: &str,
    ) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("map_declaration_to_source"); 
        let Some((file, start_pos)) = super::m5w_6::get_file_and_start_pos_from_declaration(
            &self.ls().get_program(),
            declaration,
        ) else {
            return vec![declaration.clone()];
        };
        let file_name = file.file_name.clone();

        if let Some(mapped) = self.ls().try_get_source_position(&file_name, start_pos as TextPos) {
            if let Some(source_file) = self.get_or_parse_source_file(&mapped.file_name) {
                return vec![find_closest_declaration_node(&source_file, mapped.pos as usize)];
            }
        }

        if !tsox_core::tspath::is_declaration_file_name(&file_name) {
            return vec![declaration.clone()];
        }

        let mut implementation_file = resolved_impl_file.to_string();
        if implementation_file.is_empty() {
            let dts_file_name = super::m5w_6::m5w_source_file_of_node(
                &self.ls().get_program(),
                declaration,
            )
            .map(|f| f.file_name.clone())
            .unwrap_or_default();
            let preferred_mode = self.infer_implied_node_format(&dts_file_name);
            implementation_file =
                self.find_implementation_file_from_dts_file_name(&dts_file_name, preferred_mode);
        }

        self.search_implementation_file(
            original_node,
            &implementation_file,
            &get_candidate_source_declaration_names(Some(original_node), Some(declaration)),
        )
    }

    pub fn find_implementation_file_from_dts_file_name(
        &self,
        dts_file_name: &str,
        preferred_mode: ResolutionMode,
    ) -> String { ::tsox_core::fntrace::enter("find_implementation_file_from_dts_file_name"); 
        let js_ext = tsox_tsoptions::module::mig::m3i::try_get_js_extension_for_file(
            dts_file_name,
            &self.options,
        );
        if !js_ext.is_empty() {
            let candidate = tsox_core::tspath::change_extension(dts_file_name, js_ext);
            if self.fs_file_exists(&candidate) {
                return candidate;
            }
        }

        let Some(parts) = tsox_tsoptions::modulespecifiers::get_node_module_path_parts(dts_file_name)
        else {
            return String::new();
        };

        if dts_file_name.rfind("/node_modules/") != Some(parts.top_level_node_modules_index) {
            return String::new();
        }

        let package_name_path_part =
            &dts_file_name[parts.top_level_package_name_index + 1..parts.package_root_index];
        let package_name = tsox_tsoptions::module::get_package_name_from_types_package_name(
            &tsox_tsoptions::module::unmangle_scoped_package_name(package_name_path_part),
        );
        if package_name.is_empty() {
            return String::new();
        }

        let path_to_file_in_package = &dts_file_name[parts.package_root_index + 1..];

        if !path_to_file_in_package.is_empty() {
            let specifier = format!(
                "{}/{}",
                package_name,
                tsox_core::tspath::remove_file_extension(path_to_file_in_package)
            );
            let implementation_file = self.resolve_implementation(&specifier, preferred_mode);
            if !implementation_file.is_empty() {
                return implementation_file;
            }
        }
        self.resolve_implementation(&package_name, preferred_mode)
    }

    pub fn resolve_implementation(
        &self,
        module_name: &str,
        preferred_mode: ResolutionMode,
    ) -> String { ::tsox_core::fntrace::enter("resolve_implementation"); 
        self.resolve_implementation_from(module_name, &self.resolve_from.clone(), preferred_mode)
    }

    pub fn resolve_implementation_from(
        &self,
        module_name: &str,
        resolve_from_file: &str,
        preferred_mode: ResolutionMode,
    ) -> String { ::tsox_core::fntrace::enter("resolve_implementation_from"); 
        let mut modes = vec![preferred_mode];
        if preferred_mode != ResolutionMode::ESNext {
            modes.push(ResolutionMode::ESNext);
        }
        if preferred_mode != ResolutionMode::CommonJS {
            modes.push(ResolutionMode::CommonJS);
        }

        for mode in modes {
            let resolved =
                self.resolver_resolve_module_name(module_name, resolve_from_file, mode);
            if let Some(resolved) = resolved {
                if resolved.is_resolved()
                    && !tsox_core::tspath::is_declaration_file_name(&resolved.resolved_file_name)
                {
                    return resolved.resolved_file_name;
                }
            }
        }
        String::new()
    }

    pub fn get_or_parse_source_file(&self, file_name: &str) -> Option<Arc<SourceFile>> { ::tsox_core::fntrace::enter("get_or_parse_source_file"); 
        if let Some(source_file) = (self.get_source_file)(file_name) {
            return Some(source_file);
        }
        if let Some(source_file) = self.parsed_files.borrow().get(file_name).cloned().flatten() {
            return Some(source_file);
        }
        let mut source_file: Option<Arc<SourceFile>> = None;
        if let Some(text) = self.ls().read_file(file_name) {
            let parse_options = tsox_frontend::ast::mig::m3e_2::SourceFileParseOptions {
                file_name: file_name.to_string(),
                path: self.ls().to_path(file_name).0,
                ..Default::default()
            };
            let parsed = Arc::new(tsox_compile::mig::m3l_cm_2::parse_source_file(
                &parse_options,
                text,
                tsox_core::core::mig::m3j::ensure_script_kind_from_file_name(file_name),
            ));
            tsox_checker::binder::bind_source_file(&parsed);
            source_file = Some(parsed);
        }
        self.parsed_files
            .borrow_mut()
            .insert(file_name.to_string(), source_file.clone());
        source_file
    }

    pub fn infer_implied_node_format(&self, file_name: &str) -> ResolutionMode { ::tsox_core::fntrace::enter("infer_implied_node_format"); 
        let mut package_json_type: Option<String> = None;
        if let Some(scope) = self
            .resolver_get_package_scope_for_path(&tsox_core::tspath::get_directory_path(file_name))
        {
            if scope.exists() {
                if let Some(contents) = scope.contents.as_ref() {
                    let t = &contents.header_fields.r#type;
                    if t.present {
                        package_json_type = Some(t.value.clone());
                    }
                }
            }
        }
        tsox_frontend::ast::mig::x4ast::get_implied_node_format_for_file(
            file_name,
            package_json_type.as_deref().unwrap_or(""),
        )
    }
}

pub fn core_deduplicate_strings(files: &[String]) -> Vec<String> { ::tsox_core::fntrace::enter("core_deduplicate_strings"); 
    let mut seen = HashSet::new();
    let mut result = Vec::with_capacity(files.len());
    for file in files {
        if seen.insert(file.clone()) {
            result.push(file.clone());
        }
    }
    result
}
