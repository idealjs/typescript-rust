#![allow(unused_imports, dead_code)]

//! m5l 批次 3:session.go handle* 方法族移植(归属待接线)。

use std::sync::Arc;

use super::m5l::{
    CpuProfiler, CheckerNodeParams, CheckerSignatureParams, CheckerSymbolParams, CheckerTypeParams,
    ConfigFileResponse, DiagnosticResponse, DocumentIdentifier,
    GetContextualTypeParams, GetDiagnosticsParams, GetIntrinsicTypeParams, GetParameterTypeParams,
    GetSignaturePropertyParams, GetSymbolPropertyParams, GetSymbolAtLocationParams,
    GetSourceFileParams, GetProjectDiagnosticsParams, GetTypeAtLocationParams,
    GetTypeAtLocationsParams, GetTypeAtPositionParams, GetTypeOfSymbolAtLocationParams,
    GetTypeOfSymbolParams, GetTypesAtPositionsParams, GetTypesOfSymbolsParams,
    GetTypePropertyParams, GetWidenedTypeParams, IsArrayLikeTypeParams, IsTypeAssignableToParams,
    LanguageService, ProfileParams, ProfileResult, Program, ProjectId, ProjectSession,
    ReadConfigFileParams, ReleaseParams, ResolveNameParams, Session, SignatureId,
    SignatureResponse, SnapshotId, SourceFileResponse, SymbolId, SymbolResponse, TypeId,
    TypeResponse, SnapshotData,
};
use tsox_checker::checker::checker::Checker;
use tsox_checker::checker::types::Type;
use tsox_checker::checker::types::Signature;
use tsox_frontend::ast::node_node::Node;
use tsox_frontend::ast::symbol::Symbol;
use tsox_tsoptions::vfs::FS;

impl Session {
    pub fn handle_get_this_parameter_of_signature(
        &self,
        params: &GetSignaturePropertyParams,
    ) -> Result<Option<SymbolResponse>, String> { ::tsox_core::fntrace::enter("handle_get_this_parameter_of_signature"); 
        self.resolve_symbol_property_of_signature(params, |sig| sig.this_parameter().cloned())
    }

    pub fn handle_get_true_type_of_conditional_type(
        &self,
        params: &GetTypePropertyParams,
    ) -> Result<Option<TypeResponse>, String> { ::tsox_core::fntrace::enter("handle_get_true_type_of_conditional_type"); 
        let setup = self.setup_checker(params.snapshot, &params.project)?;
        let t = setup.sd.resolve_type_handle(&params.project, params.r#type)?;
        match setup.checker.get_true_type_of_conditional_type(&t) {
            None => Ok(None),
            Some(tt) => Ok(setup.sd.new_type_response(&params.project, &tt)),
        }
    }

    pub fn handle_get_type_arguments(
        &self,
        params: &CheckerTypeParams,
    ) -> Result<Option<Vec<TypeResponse>>, String> { ::tsox_core::fntrace::enter("handle_get_type_arguments"); 
        let setup = self.setup_checker(params.snapshot, &params.project)?;
        let t = setup.resolve_type_handle(params.r#type)?;
        let type_args = setup.checker.get_type_arguments(&t);
        if type_args.is_empty() {
            return Ok(None);
        }
        Ok(Some(
            type_args
                .iter()
                .filter_map(|ta| setup.new_type_response(ta))
                .collect(),
        ))
    }

    pub fn handle_get_type_at_location(
        &self,
        params: &GetTypeAtLocationParams,
    ) -> Result<Option<TypeResponse>, String> { ::tsox_core::fntrace::enter("handle_get_type_at_location"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let node = setup.sd.resolve_node_handle(&setup.program, &params.location)?;
        let t = setup.checker.get_type_at_location(node.as_ref().unwrap());
        Ok(setup.new_type_response(&t))
    }

    pub fn handle_get_type_at_locations(
        &self,
        params: &GetTypeAtLocationsParams,
    ) -> Result<Vec<TypeResponse>, String> { ::tsox_core::fntrace::enter("handle_get_type_at_locations"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let mut results = Vec::with_capacity(params.locations.len());
        for loc in &params.locations {
            let node = setup.sd.resolve_node_handle(&setup.program, loc)?;
            let t = setup.checker.get_type_at_location(node.as_ref().unwrap());
            if let Some(resp) = setup.new_type_response(&t) {
                results.push(resp);
            }
        }
        Ok(results)
    }

    pub fn handle_get_type_at_position(
        &self,
        params: &GetTypeAtPositionParams,
    ) -> Result<Option<TypeResponse>, String> { ::tsox_core::fntrace::enter("handle_get_type_at_position"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let source_file = setup
            .program
            .get_source_file(&params.file.to_file_name())
            .ok_or_else(|| client_error(format!("source file not found: {}", params.file)))?;
        let position_map = tsox_frontend::ast::positionmap::compute_position_map(&source_file.text);
        let node = tsox_frontend::astnav::get_touching_property_name(
            &source_file.node,
            position_map.utf16_to_utf8(params.position as usize),
        );
        let Some(node) = node else { return Ok(None) };
        let t = setup.checker.get_type_at_location(&node);
        Ok(setup.new_type_response(&t))
    }

    pub fn handle_get_type_from_type_node(
        &self,
        params: &GetTypeAtLocationParams,
    ) -> Result<Option<TypeResponse>, String> { ::tsox_core::fntrace::enter("handle_get_type_from_type_node"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let node = setup.sd.resolve_node_handle(&setup.program, &params.location)?;
        let t = setup
            .checker
            .get_type_from_type_node(node.as_ref().unwrap());
        Ok(setup.new_type_response(&t))
    }

    pub fn handle_get_type_of_symbol(
        &self,
        params: &GetTypeOfSymbolParams,
    ) -> Result<Option<TypeResponse>, String> { ::tsox_core::fntrace::enter("handle_get_type_of_symbol"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let symbol = setup.resolve_symbol_handle(params.symbol)?;
        let t = setup.checker.get_type_of_symbol(&symbol);
        Ok(setup.new_type_response(&t))
    }

    pub fn handle_get_type_of_symbol_at_location(
        &self,
        params: &GetTypeOfSymbolAtLocationParams,
    ) -> Result<Option<TypeResponse>, String> { ::tsox_core::fntrace::enter("handle_get_type_of_symbol_at_location"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let symbol = setup.resolve_symbol_handle(params.symbol)?;
        let node = setup.sd.resolve_node_handle(&setup.program, &params.location)?;
        let t = setup
            .checker
            .get_type_of_symbol_at_location(&symbol, node.as_ref().unwrap());
        Ok(setup.new_type_response(&t))
    }

    pub fn handle_get_type_parameter_at_position(
        &self,
        params: &GetParameterTypeParams,
    ) -> Result<Option<TypeResponse>, String> { ::tsox_core::fntrace::enter("handle_get_type_parameter_at_position"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let sig = setup.resolve_signature_handle(params.signature)?;
        if params.index < 0 {
            return Err(client_error("invalid parameter index"));
        }
        let t = setup
            .checker
            .get_type_parameter_at_position(&sig, params.index as usize);
        Ok(setup.new_type_response(&t))
    }

    pub fn handle_get_type_parameters_of_signature(
        &self,
        params: &GetSignaturePropertyParams,
    ) -> Result<Option<Vec<TypeResponse>>, String> { ::tsox_core::fntrace::enter("handle_get_type_parameters_of_signature"); 
        self.resolve_type_array_property_of_signature(params, |sig| sig.type_parameters.clone())
    }

    pub fn handle_get_type_parameters_of_type(
        &self,
        params: &GetTypePropertyParams,
    ) -> Result<Option<Vec<TypeResponse>>, String> { ::tsox_core::fntrace::enter("handle_get_type_parameters_of_type"); 
        self.resolve_type_array_property_of_type(params, |t| {
            t.as_interface_type()
                .map(|it| it.type_parameters().to_vec())
                .unwrap_or_default()
        })
    }

    pub fn handle_get_type_predicate_of_signature(
        &self,
        params: &CheckerSignatureParams,
    ) -> Result<Option<TypePredicateResponse>, String> { ::tsox_core::fntrace::enter("handle_get_type_predicate_of_signature"); 
        let setup = self.setup_checker(params.snapshot, &params.project)?;
        let sig = setup.resolve_signature_handle(params.signature)?;
        let pred = match setup.checker.get_type_predicate_of_signature(&sig) {
            None => return Ok(None),
            Some(pred) => pred,
        };
        let mut resp = TypePredicateResponse {
            kind: pred.kind() as i32,
            parameter_index: pred.parameter_index(),
            parameter_name: pred.parameter_name().to_string(),
            r#type: None,
        };
        if let Some(t) = pred.ty() {
            resp.r#type = setup.new_type_response(t);
        }
        Ok(Some(resp))
    }

    pub fn handle_get_types_at_positions(
        &self,
        params: &GetTypesAtPositionsParams,
    ) -> Result<Vec<TypeResponse>, String> { ::tsox_core::fntrace::enter("handle_get_types_at_positions"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let source_file = setup
            .program
            .get_source_file(&params.file.to_file_name())
            .ok_or_else(|| client_error(format!("source file not found: {}", params.file)))?;
        let position_map = tsox_frontend::ast::positionmap::compute_position_map(&source_file.text);
        let mut results = Vec::with_capacity(params.positions.len());
        for pos in &params.positions {
            let node = tsox_frontend::astnav::get_touching_property_name(
                &source_file.node,
                position_map.utf16_to_utf8(*pos as usize),
            );
            if let Some(node) = node {
                let t = setup.checker.get_type_at_location(&node);
                if let Some(resp) = setup.new_type_response(&t) {
                    results.push(resp);
                }
            }
        }
        Ok(results)
    }

    pub fn handle_get_types_of_symbols(
        &self,
        params: &GetTypesOfSymbolsParams,
    ) -> Result<Vec<TypeResponse>, String> { ::tsox_core::fntrace::enter("handle_get_types_of_symbols"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let mut results = Vec::with_capacity(params.symbols.len());
        for sym_handle in &params.symbols {
            let symbol = setup.resolve_symbol_handle(*sym_handle)?;
            let t = setup.checker.get_type_of_symbol(&symbol);
            if let Some(resp) = setup.new_type_response(&t) {
                results.push(resp);
            }
        }
        Ok(results)
    }

    pub fn handle_get_types_of_type(
        &self,
        params: &GetTypePropertyParams,
    ) -> Result<Option<Vec<TypeResponse>>, String> { ::tsox_core::fntrace::enter("handle_get_types_of_type"); 
        self.resolve_type_array_property_of_type(params, |t| t.types().unwrap_or_default().to_vec())
    }

    pub fn handle_get_well_known_signatures(
        &self,
        params: &GetIntrinsicTypeParams,
    ) -> Result<WellKnownSignaturesResponse, String> { ::tsox_core::fntrace::enter("handle_get_well_known_signatures"); 
        let setup = self.setup_checker(params.snapshot, &params.project)?;
        let sig = match setup.checker.get_unknown_signature() {
            Some(sig) => sig,
            None => return Err(client_error("unknown signature is not available")),
        };
        Ok(WellKnownSignaturesResponse {
            unknown: setup.sd.register_signature(&setup.project_id, &sig),
        })
    }

    pub fn handle_get_well_known_symbols(
        &self,
        params: &GetIntrinsicTypeParams,
    ) -> Result<WellKnownSymbolsResponse, String> { ::tsox_core::fntrace::enter("handle_get_well_known_symbols"); 
        let setup = self.setup_checker(params.snapshot, &params.project)?;
        let unknown = match setup.checker.get_unknown_symbol() {
            Some(sym) => sym,
            None => return Err(client_error("unknown symbol is not available")),
        };
        let undefined = match setup.checker.get_undefined_symbol() {
            Some(sym) => sym,
            None => return Err(client_error("undefined symbol is not available")),
        };
        let arguments = match setup.checker.get_arguments_symbol() {
            Some(sym) => sym,
            None => return Err(client_error("arguments symbol is not available")),
        };
        Ok(WellKnownSymbolsResponse {
            unknown: setup.sd.register_symbol(&unknown, &setup.project_id),
            undefined: setup.sd.register_symbol(&undefined, &setup.project_id),
            arguments: setup.sd.register_symbol(&arguments, &setup.project_id),
        })
    }

    pub fn handle_get_widened_type(
        &self,
        params: &GetWidenedTypeParams,
    ) -> Result<Option<TypeResponse>, String> { ::tsox_core::fntrace::enter("handle_get_widened_type"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let t = setup.resolve_type_handle(params.r#type)?;
        let widened = setup.checker.get_widened_type(&t);
        Ok(setup.new_type_response(&widened))
    }

    pub fn handle_initialize(&self) -> Result<InitializeResponse, String> { ::tsox_core::fntrace::enter("handle_initialize"); 
        Ok(InitializeResponse {
            use_case_sensitive_file_names: self
                .project_session
                .fs()
                .map(|fs| fs.use_case_sensitive_file_names())
                .unwrap_or(true),
            current_directory: self.project_session.get_current_directory(),
        })
    }

    pub fn handle_is_array_like_type(
        &self,
        params: &IsArrayLikeTypeParams,
    ) -> Result<bool, String> { ::tsox_core::fntrace::enter("handle_is_array_like_type"); 
        let setup = self.setup_checker(params.snapshot, &params.project)?;
        let t = setup.resolve_type_handle(params.r#type)?;
        Ok(setup.checker.is_array_like_type(&t))
    }

    pub fn handle_is_array_type(&self, params: &CheckerTypeParams) -> Result<bool, String> { ::tsox_core::fntrace::enter("handle_is_array_type"); 
        let setup = self.setup_checker(params.snapshot, &params.project)?;
        let t = setup.resolve_type_handle(params.r#type)?;
        Ok(setup.checker.is_array_type(&t))
    }

    pub fn handle_is_context_sensitive(
        &self,
        params: &GetContextualTypeParams,
    ) -> Result<bool, String> { ::tsox_core::fntrace::enter("handle_is_context_sensitive"); 
        let setup = self.setup_checker(params.snapshot, &params.project)?;
        let node = setup.sd.resolve_node_handle(&setup.program, &params.location)?;
        let Some(node) = node else { return Ok(false) };
        Ok(setup.checker.is_context_sensitive(&node))
    }

    pub fn handle_is_readonly_symbol(
        &self,
        params: &CheckerSymbolParams,
    ) -> Result<bool, String> { ::tsox_core::fntrace::enter("handle_is_readonly_symbol"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let symbol = setup.resolve_symbol_handle(params.symbol)?;
        Ok(setup.checker.is_readonly_symbol(&symbol))
    }

    pub fn handle_is_type_assignable_to(
        &self,
        params: &IsTypeAssignableToParams,
    ) -> Result<bool, String> { ::tsox_core::fntrace::enter("handle_is_type_assignable_to"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let source = setup.resolve_type_handle(params.source)?;
        let target = setup.resolve_type_handle(params.target)?;
        Ok(setup.checker.is_type_assignable_to(&source, &target))
    }

    pub fn handle_method_get_target_symbol(
        &self,
        params: &CheckerSymbolParams,
    ) -> Result<Option<SymbolResponse>, String> { ::tsox_core::fntrace::enter("handle_method_get_target_symbol"); 
        let setup = self.setup_checker(params.snapshot, &params.project)?;
        let symbol = setup.resolve_symbol_handle(params.symbol)?;
        Ok(setup.new_symbol_response(&setup.checker.get_target_symbol(&symbol)))
    }

    pub fn handle_parse_command_line(
        &self,
        params: &ParseCommandLineParams,
    ) -> Result<ConfigFileResponse, String> { ::tsox_core::fntrace::enter("handle_parse_command_line"); 
        Ok(new_config_file_response(&tsox_tsoptions::tsoptions::parse_command_line(
            &params.command_line,
            &self.project_session.get_current_directory(),
            self.project_session.fs().map(|fs| &**fs),
        )))
    }

    pub fn handle_parse_config_file(
        &self,
        params: &ParseConfigFileParams,
    ) -> Result<ConfigFileResponse, String> { ::tsox_core::fntrace::enter("handle_parse_config_file"); 
        let config_file_name = params
            .file
            .to_absolute_file_name(&self.project_session.get_current_directory());
        let config_file_content = self
            .project_session
            .fs()
            .and_then(|fs| fs.read_file(&config_file_name))
            .ok_or_else(|| client_error(format!("could not read file {:?}", config_file_name)))?;
        let config_dir = tsox_core::tspath::get_directory_path(&config_file_name);
        let ts_config_source_file = tsox_tsoptions::mig::m5i2::new_tsconfig_source_file_from_file_path(
            &config_file_name,
            &self.to_path(&config_file_name),
            &config_file_content,
        );
        let host = new_parse_config_host(&self.project_session);
        let parsed_command_line =
            tsox_tsoptions::mig::m5j_3::parse_json_source_file_config_file_content(
                &ts_config_source_file,
                &host,
                &config_dir,
                None,
                None,
                config_file_name.as_str(),
                &[],
                None,
            );
        Ok(new_config_file_response(&parsed_command_line))
    }

    pub fn handle_parse_json_config_file_content(
        &self,
        params: &ParseJsonConfigFileContentParams,
    ) -> Result<ConfigFileResponse, String> { ::tsox_core::fntrace::enter("handle_parse_json_config_file_content"); 
        if params.config_directory.is_none() == params.config_file_name.is_none() {
            return Err(client_error(
                "exactly one of configDirectory or configFileName is required",
            ));
        }
        let (base_path, config_file_name) = match &params.config_directory {
            Some(dir) => (
                tsox_core::tspath::get_normalized_absolute_path(
                    dir,
                    &self.project_session.get_current_directory(),
                ),
                String::new(),
            ),
            None => {
                let name = params
                    .config_file_name
                    .as_ref()
                    .unwrap()
                    .to_absolute_file_name(&self.project_session.get_current_directory());
                let base = tsox_core::tspath::get_directory_path(&name);
                (base, name)
            }
        };
        let host = new_parse_config_host(&self.project_session);
        let parsed_command_line = tsox_tsoptions::mig::m5i2_4::parse_json_config_file_content(
            &params.json,
            &host,
            &base_path,
            None,
            config_file_name.as_str(),
            &[],
            None,
        );
        Ok(new_config_file_response(&parsed_command_line))
    }

    pub fn handle_print_node(&self, params: &PrintNodeParams) -> Result<String, String> { ::tsox_core::fntrace::enter("handle_print_node"); 
        let data = base64_decode(&params.data)
            .map_err(|e| client_error(format!("invalid base64 data: {}", e)))?;
        let node = super::m5j_decoder::decode_nodes(&data)
            .map_err(|e| client_error(format!("failed to decode AST: {}", e)))?;
        let mut p = tsox_frontend::format::mig::m4o_2::new_printer(
            tsox_frontend::format::mig::m4o_2::PrinterOptions {
                remove_comments: false,
                new_line: tsox_core::core::compiler_options_kinds::NewLineKind::default(),
                omit_trailing_semicolon: false,
                no_emit_helpers: false,
                target: tsox_core::core::compiler_options_kinds::ScriptTarget::default(),
                source_map: false,
                inline_source_map: false,
                inline_sources: false,
                omit_brace_source_map_positions: false,
                only_print_jsdoc_style: false,
                never_ascii_escape: params.never_ascii_escape,
                preserve_source_newlines: params.preserve_source_newlines,
                terminate_unterminated_literals: params.terminate_unterminated_literals,
            },
            tsox_frontend::format::mig::m4o_2::PrintHandlers {
                has_global_name: None,
                map_source_position: None,
                on_before_emit_node: None,
                on_after_emit_node: None,
                on_before_emit_node_list: None,
                on_after_emit_node_list: None,
                on_before_emit_token: None,
                on_after_emit_token: None,
            },
            tsox_frontend::format::mig::m4o_2::EmitContext::default(),
        );
        Ok(p.emit(&node, None))
    }

    pub fn handle_read_config_file(
        &self,
        params: &ReadConfigFileParams,
    ) -> Result<ReadConfigFileResponse, String> { ::tsox_core::fntrace::enter("handle_read_config_file"); 
        let config_file_name = params
            .file
            .to_absolute_file_name(&self.project_session.get_current_directory());
        let Some(config_file_content) = self
            .project_session
            .fs()
            .and_then(|fs| fs.read_file(&config_file_name))
        else {
            return Ok(ReadConfigFileResponse {
                config: Default::default(),
                error: Some(new_diagnostic_response(
                    &tsox_frontend::ast::mig::m3d_2::new_compiler_diagnostic(
                        tsox_core::diagnostics::messages_generated::CANNOT_READ_FILE_0,
                        vec![config_file_name],
                    ),
                )),
            });
        };
        let (config, parse_errors) = tsox_tsoptions::mig::m5i2::parse_config_file_text_to_json(
            &config_file_name,
            &self.to_path(&config_file_name),
            &config_file_content,
        );
        let config = match config {
            Some(serde_json::Value::Object(map)) => map,
            _ => Default::default(),
        };
        let mut response = ReadConfigFileResponse {
            config,
            error: None,
        };
        if !parse_errors.is_empty() {
            response.error = Some(new_diagnostic_response(&parse_errors[0]));
        }
        Ok(response)
    }

    pub fn handle_release(&self, params: &ReleaseParams) -> Result<bool, String> { ::tsox_core::fntrace::enter("handle_release"); 
        if params.snapshot == 0 {
            return Err(client_error("empty handle"));
        }
        self.release_snapshot(params.snapshot)?;
        Ok(true)
    }

    pub fn handle_resolve_name(
        &self,
        params: &ResolveNameParams,
    ) -> Result<Option<SymbolResponse>, String> { ::tsox_core::fntrace::enter("handle_resolve_name"); 
        let setup = self.setup_checker(params.snapshot, &params.project)?;
        let location =
            setup.resolve_location(&params.location, params.file.as_ref(), params.position)?;
        let location =
            location.ok_or_else(|| client_error("resolve_name requires a location"))?;
        let symbol = setup.checker.resolve_name(
            &params.name,
            &location,
            tsox_frontend::ast::SymbolFlags::from_bits_retain(params.meaning),
            params.exclude_globals,
        );
        match symbol {
            None => Ok(None),
            Some(symbol) => Ok(setup.new_symbol_response(&symbol)),
        }
    }

    pub fn handle_save_heap_profile(
        &self,
        params: &ProfileParams,
    ) -> Result<ProfileResult, String> { ::tsox_core::fntrace::enter("handle_save_heap_profile"); 
        if params.dir.is_empty() {
            return Err(client_error("dir is required"));
        }
        let file_path = tsox_core::pprof::mig::m6a::save_heap_profile(&params.dir)
            .map_err(|e| client_error(format!("failed to save heap profile: {}", e)))?;
        Ok(ProfileResult { file: file_path })
    }

    pub fn handle_selected_files_emit(
        &self,
        params: &SelectedFilesEmitParams,
        emit_only: tsox_compile::mig::m4v::EmitOnly,
    ) -> Result<EmitOutputResponse, String> { ::tsox_core::fntrace::enter("handle_selected_files_emit"); 
        let program = self.get_emit_program(params.snapshot, &params.project)?;
        if params.files.is_none() {
            return Err(client_error("files is required"));
        }
        let mut target_source_files = Vec::with_capacity(params.files.as_ref().unwrap().len());
        for file in params.files.as_ref().unwrap() {
            let source_file = self.resolve_optional_source_file(&program, Some(file))?;
            target_source_files.push(source_file);
        }
        emit_selected_files(&program, &target_source_files, emit_only, params)
    }

    pub fn handle_signature_to_signature_declaration(
        &self,
        params: &SignatureToSignatureDeclarationParams,
    ) -> Result<Option<SourceFileResponse>, String> { ::tsox_core::fntrace::enter("handle_signature_to_signature_declaration"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let sig = setup.resolve_signature_handle(params.signature)?;
        let mut enclosing_declaration: Option<Arc<Node>> = None;
        if !params.location.is_empty() {
            enclosing_declaration = setup.sd.resolve_node_handle(&setup.program, &params.location)?;
        }
        let node = setup.checker.signature_to_signature_declaration(
            &sig,
            unsafe { std::mem::transmute::<i16, tsox_frontend::ast::SyntaxKind>(params.kind as i16) },
            enclosing_declaration.as_ref(),
            tsox_checker::checker::symboltracker::NodeBuilderFlags::from_bits_retain(params.flags),
        );
        let (data, _) = super::m5j_encoder::encode_tree(&node, None)
            .map_err(|e| format!("failed to encode signature declaration: {}", e))?;
        if self.use_binary_responses {
            return Ok(None);
        }
        Ok(Some(SourceFileResponse {
            data: base64_encode(&data),
        }))
    }

    pub fn handle_start_cpu_profile(
        &self,
        params: &ProfileParams,
    ) -> Result<(), String> { ::tsox_core::fntrace::enter("handle_start_cpu_profile"); 
        if params.dir.is_empty() {
            return Err(client_error("dir is required"));
        }
        self.cpu_profiler
            .start_cpu_profile(&params.dir)
            .map_err(|e| client_error(format!("failed to start CPU profile: {}", e)))
    }

    pub fn handle_stop_cpu_profile(&self) -> Result<ProfileResult, String> { ::tsox_core::fntrace::enter("handle_stop_cpu_profile"); 
        let file_path = self
            .cpu_profiler
            .stop_cpu_profile()
            .map_err(|e| client_error(format!("failed to stop CPU profile: {}", e)))?;
        Ok(ProfileResult { file: file_path })
    }
}

fn client_error(msg: impl std::fmt::Display) -> String { ::tsox_core::fntrace::enter("client_error"); 
    format!("client error: {}", msg)
}

fn new_parse_config_host(session: &ProjectSession) -> tsox_tsoptions::mig::m5j_2::ParseConfigHost { ::tsox_core::fntrace::enter("new_parse_config_host"); 
    tsox_tsoptions::mig::m5j_2::ParseConfigHost {
        fs: session
            .fs()
            .cloned()
            .unwrap_or_else(|| Arc::new(tsox_tsoptions::vfs::InMemoryFS::new())),
        current_directory: session.get_current_directory(),
    }
}

#[derive(serde::Serialize)]
pub struct TypePredicateResponse {
    pub kind: i32,
    pub parameter_index: i32,
    pub parameter_name: String,
    pub r#type: Option<TypeResponse>,
}

#[derive(serde::Serialize)]
pub struct InitializeResponse {
    pub use_case_sensitive_file_names: bool,
    pub current_directory: String,
}

#[derive(serde::Serialize)]
pub struct WellKnownSymbolsResponse {
    pub unknown: SymbolId,
    pub undefined: SymbolId,
    pub arguments: SymbolId,
}

#[derive(serde::Serialize)]
pub struct WellKnownSignaturesResponse {
    pub unknown: SignatureId,
}

#[derive(serde::Serialize)]
pub struct ReadConfigFileResponse {
    pub config: serde_json::Map<String, serde_json::Value>,
    pub error: Option<DiagnosticResponse>,
}

#[derive(serde::Deserialize)]
pub struct ParseCommandLineParams {
    pub command_line: Vec<String>,
}

#[derive(serde::Deserialize)]
pub struct ParseConfigFileParams {
    pub file: DocumentIdentifier,
}

#[derive(serde::Deserialize)]
pub struct ParseJsonConfigFileContentParams {
    pub json: serde_json::Value,
    pub config_directory: Option<String>,
    pub config_file_name: Option<DocumentIdentifier>,
}

#[derive(serde::Deserialize)]
pub struct PrintNodeParams {
    pub data: String,
    pub preserve_source_newlines: bool,
    pub never_ascii_escape: bool,
    pub terminate_unterminated_literals: bool,
}

#[derive(serde::Deserialize)]
pub struct SelectedFilesEmitParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub files: Option<Vec<DocumentIdentifier>>,
}

#[derive(serde::Deserialize)]
pub struct SignatureToSignatureDeclarationParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub signature: SignatureId,
    pub location: NodeHandle,
    pub kind: u32,
    pub flags: u32,
}

#[derive(serde::Serialize)]
pub struct EmitOutputResponse;

pub fn json_value_to_any(value: &serde_json::Value) -> serde_json::Value { ::tsox_core::fntrace::enter("json_value_to_any"); 
    value.clone()
}

pub fn base64_encode(data: &[u8]) -> String { ::tsox_core::fntrace::enter("base64_encode"); 
    data.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn base64_decode(s: &str) -> Result<Vec<u8>, String> { ::tsox_core::fntrace::enter("base64_decode"); 
    if s.len() % 2 != 0 {
        return Err("odd length".to_string());
    }
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

fn new_config_file_response(
    _cmd: &tsox_tsoptions::tsoptions::ParsedCommandLine,
) -> ConfigFileResponse { ::tsox_core::fntrace::enter("new_config_file_response"); 
    unimplemented!()
}

fn new_diagnostic_response(_diag: &tsox_frontend::ast::Diagnostic) -> DiagnosticResponse { ::tsox_core::fntrace::enter("new_diagnostic_response"); 
    unimplemented!()
}

fn emit_selected_files(
    _program: &Program,
    _files: &[Option<Arc<tsox_frontend::ast::SourceFile>>],
    _emit_only: tsox_compile::mig::m4v::EmitOnly,
    _params: &SelectedFilesEmitParams,
) -> Result<EmitOutputResponse, String> { ::tsox_core::fntrace::enter("emit_selected_files"); 
    unimplemented!()
}

use super::m5l::NodeHandle;

impl Session {
    pub fn release_snapshot(&self, _handle: SnapshotId) -> Result<(), String> { ::tsox_core::fntrace::enter("release_snapshot"); 
        unimplemented!()
    }

    pub fn get_emit_program(
        &self,
        _snapshot: SnapshotId,
        _project: &ProjectId,
    ) -> Result<Program, String> { ::tsox_core::fntrace::enter("get_emit_program"); 
        unimplemented!()
    }
}
