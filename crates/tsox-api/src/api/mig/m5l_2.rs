#![allow(unused_imports, dead_code)]

//! m5l 批次 2:session.go handle* 方法族移植(归属待接线)。

use std::sync::Arc;

use super::m5l::{
    client_error, CheckerNodeParams, CheckerSetup, CheckerSignatureParams, CheckerSymbolParams,
    CheckerTypeParams, DiagnosticResponse, DocumentIdentifier, GetDiagnosticsParams,
    GetParameterTypeParams, GetProjectDiagnosticsParams, GetPropertyOfTypeParams,
    GetResolvedSignatureParams, GetSignaturePropertyParams, GetSignaturesOfTypeParams,
    GetSourceFileNamesParams, GetSourceFileParams, GetSymbolAtLocationParams,
    GetSymbolAtPositionParams, GetSymbolPropertyParams, GetSymbolsAtLocationsParams,
    GetSymbolsAtPositionsParams, GetSymbolsInScopeParams, GetSymbolsOfSourceFilesParams,
    GetTypeAtLocationParams, GetTypeAtLocationsParams, GetTypeAtPositionParams,
    GetTypeOfSymbolAtLocationParams, GetTypeOfSymbolParams, GetTypePropertyParams,
    GetTypesAtPositionsParams, GetTypesOfSymbolsParams, Program, ProjectId, Session, SignatureId,
    SignatureResponse, SnapshotData, SnapshotId, SymbolId, SymbolResponse, TypeId, TypePredicateResponse,
    TypeResponse,
};
use tsox_checker::checker::types::Type;
use tsox_checker::checker::types::Signature;
use tsox_frontend::ast::node_node::Node;
use tsox_frontend::ast::symbol::Symbol;

#[derive(serde::Deserialize)]
pub struct GetSymbolOfSourceFileParams {
    pub snapshot: SnapshotId,
    pub project: ProjectId,
    pub file: DocumentIdentifier,
}

#[derive(serde::Serialize)]
pub struct ReferencedSymbolEntry {
    pub definition: NodeHandle,
    pub symbol: Option<SymbolResponse>,
    pub references: Vec<NodeHandle>,
}

#[derive(serde::Serialize)]
pub struct SignatureUsageResponse {
    pub name: NodeHandle,
    pub call: Option<NodeHandle>,
}

#[derive(serde::Serialize)]
pub struct SourceFileMetadata {
    pub is_default_library: bool,
    pub is_from_external_library: bool,
    pub package_json_type: String,
    pub package_json_directory: String,
    pub implied_node_format: i32,
}

impl Session {
    pub fn handle_get_non_nullable_type(
        &self,
        params: &GetTypePropertyParams,
    ) -> Result<Option<TypeResponse>, String> { ::tsox_core::fntrace::enter("handle_get_non_nullable_type"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let t = setup.resolve_type_handle(params.r#type)?;
        let result = setup.checker.get_non_nullable_type(&t);
        Ok(setup.new_type_response(&result))
    }

    pub fn handle_get_object_type_of_type(
        &self,
        params: &GetTypePropertyParams,
    ) -> Result<Option<TypeResponse>, String> { ::tsox_core::fntrace::enter("handle_get_object_type_of_type"); 
        let sd = self.get_snapshot_data(params.snapshot)?;
        let t = sd.resolve_type_handle(&params.project, params.r#type)?;
        match t
            .as_indexed_access_type()
            .and_then(|d| d.object_type().cloned())
        {
            None => Ok(None),
            Some(result) => Ok(sd.new_type_response(&params.project, &result)),
        }
    }

    pub fn handle_get_outer_type_parameters_of_type(
        &self,
        params: &GetTypePropertyParams,
    ) -> Result<Option<Vec<TypeResponse>>, String> { ::tsox_core::fntrace::enter("handle_get_outer_type_parameters_of_type"); 
        let sd = self.get_snapshot_data(params.snapshot)?;
        let t = sd.resolve_type_handle(&params.project, params.r#type)?;
        let types = t
            .as_interface_type()
            .map(|d| d.outer_type_parameters().to_vec())
            .unwrap_or_default();
        if types.is_empty() {
            return Ok(None);
        }
        Ok(Some(
            types
                .iter()
                .filter_map(|sub| sd.new_type_response(&params.project, sub))
                .collect(),
        ))
    }

    pub fn handle_get_parameter_type(
        &self,
        params: &GetParameterTypeParams,
    ) -> Result<Option<TypeResponse>, String> { ::tsox_core::fntrace::enter("handle_get_parameter_type"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let sig = setup.resolve_signature_handle(params.signature)?;
        if params.index < 0 {
            return Err(client_error("invalid parameter index"));
        }
        let result = setup
            .checker
            .get_type_at_position(&sig, params.index as usize);
        Ok(setup.new_type_response(&result))
    }

    pub fn handle_get_parameters_of_signature(
        &self,
        params: &GetSignaturePropertyParams,
    ) -> Result<Option<Vec<SymbolResponse>>, String> { ::tsox_core::fntrace::enter("handle_get_parameters_of_signature"); 
        let sd = self.get_snapshot_data(params.snapshot)?;
        let sig = sd.resolve_signature_handle(&params.project, params.signature)?;
        let symbols = sig.parameters();
        if symbols.is_empty() {
            return Ok(None);
        }
        Ok(Some(
            symbols
                .iter()
                .filter_map(|sym| sd.new_symbol_response(sym, &params.project))
                .collect(),
        ))
    }

    pub fn handle_get_parent_of_symbol(
        &self,
        params: &GetSymbolPropertyParams,
    ) -> Result<Option<SymbolResponse>, String> { ::tsox_core::fntrace::enter("handle_get_parent_of_symbol"); 
        self.resolve_symbol_property_of_symbol(params, |sym| sym.parent())
    }

    pub fn handle_get_program_diagnostics(
        &self,
        params: &GetProjectDiagnosticsParams,
    ) -> Result<Vec<DiagnosticResponse>, String> { ::tsox_core::fntrace::enter("handle_get_program_diagnostics"); 
        let sd = self.get_snapshot_data(params.snapshot)?;
        let program = sd.get_program(&params.project)?;
        let diags = program.get_program_diagnostics();
        Ok(new_diagnostic_responses(&diags))
    }

    pub fn handle_get_properties_of_type(
        &self,
        params: &CheckerTypeParams,
    ) -> Result<Option<Vec<SymbolResponse>>, String> { ::tsox_core::fntrace::enter("handle_get_properties_of_type"); 
        let setup = self.setup_checker(params.snapshot, &params.project)?;
        let t = setup.resolve_type_handle(params.r#type)?;
        let props = setup.checker.get_properties_of_type(&t);
        if props.is_empty() {
            return Ok(None);
        }
        Ok(Some(
            props
                .iter()
                .filter_map(|prop| setup.new_symbol_response(prop))
                .collect(),
        ))
    }

    pub fn handle_get_property_of_type(
        &self,
        params: &GetPropertyOfTypeParams,
    ) -> Result<Option<SymbolResponse>, String> { ::tsox_core::fntrace::enter("handle_get_property_of_type"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let t = setup.resolve_type_handle(params.r#type)?;
        let prop = setup.checker.get_property_of_type(&t, &params.name);
        match prop {
            None => Ok(None),
            Some(prop) => Ok(setup.new_symbol_response(&prop)),
        }
    }

    pub fn handle_get_reduced_type(
        &self,
        params: &GetTypePropertyParams,
    ) -> Result<Option<TypeResponse>, String> { ::tsox_core::fntrace::enter("handle_get_reduced_type"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let t = setup.resolve_type_handle(params.r#type)?;
        let result = setup.checker.get_reduced_type(&t);
        Ok(setup.new_type_response(&result))
    }

    pub fn handle_get_referenced_symbols_for_node(
        &self,
        params: &GetReferencedSymbolsForNodeParams,
    ) -> Result<Option<Vec<ReferencedSymbolEntry>>, String> { ::tsox_core::fntrace::enter("handle_get_referenced_symbols_for_node"); 
        let sd = self.get_snapshot_data(params.snapshot)?;
        let program = sd.get_program(&params.project)?;
        let node = sd.resolve_node_handle(&program, &params.node)?;
        let Some(node) = node else { return Ok(None) };
        let lang_svc = self.setup_language_service(&sd, &program, &params.project, "")?;
        let source_files = program.get_source_files();
        let entries = lang_svc.get_referenced_symbols_for_node(
            &mut core_context(),
            params.position,
            &node,
            &source_files,
        );
        let Some(entries) = entries else {
            return Ok(None);
        };
        let mut result = Vec::new();
        for entry in &entries {
            let Some(def_node) = entry.definition_node.as_ref() else {
                continue;
            };
            let refs: Vec<NodeHandle> = entry
                .references
                .iter()
                .filter(|r| r.is_node_entry)
                .filter_map(|r| r.node.as_ref().map(|n| sd.node_handle_from(n)))
                .collect();
            let mut re = ReferencedSymbolEntry {
                definition: sd.node_handle_from(def_node),
                references: refs,
                symbol: None,
            };
            if let Some(sym) = entry.definition_symbol.as_ref() {
                re.symbol = sd.new_symbol_response(sym, &params.project);
            }
            result.push(re);
        }
        Ok(Some(result))
    }

    pub fn handle_get_references_to_symbol_in_file(
        &self,
        params: &GetReferencesToSymbolInFileParams,
    ) -> Result<Vec<NodeHandle>, String> { ::tsox_core::fntrace::enter("handle_get_references_to_symbol_in_file"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let symbol = setup.resolve_symbol_handle(params.symbol)?;
        let source_file = setup
            .program
            .get_source_file(&params.file.to_file_name())
            .ok_or_else(|| client_error(format!("source file not found: {}", params.file)))?;
        let nodes = setup
            .checker
            .get_references_to_symbol_in_file(&source_file, &symbol);
        Ok(nodes.iter().map(|n| setup.sd.node_handle_from(n)).collect())
    }

    pub fn handle_get_regular_type_of_type(
        &self,
        params: &GetTypePropertyParams,
    ) -> Result<Option<TypeResponse>, String> { ::tsox_core::fntrace::enter("handle_get_regular_type_of_type"); 
        let sd = self.get_snapshot_data(params.snapshot)?;
        let t = sd.resolve_type_handle(&params.project, params.r#type)?;
        match t
            .as_literal_type()
            .and_then(|d| d.regular_type().cloned())
        {
            None => Ok(None),
            Some(result) => Ok(sd.new_type_response(&params.project, &result)),
        }
    }

    pub fn handle_get_resolved_signature(
        &self,
        params: &GetResolvedSignatureParams,
    ) -> Result<Option<SignatureResponse>, String> { ::tsox_core::fntrace::enter("handle_get_resolved_signature"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let node = setup
            .sd
            .resolve_node_handle(&setup.program, &params.location)?;
        let Some(node) = node else { return Ok(None) };
        let sig = setup.checker.get_resolved_signature(&node);
        Ok(match sig {
            None => None,
            Some(sig) => setup.new_signature_response(&sig),
        })
    }

    pub fn handle_get_rest_type_of_signature(
        &self,
        params: &CheckerSignatureParams,
    ) -> Result<Option<TypeResponse>, String> { ::tsox_core::fntrace::enter("handle_get_rest_type_of_signature"); 
        let setup = self.setup_checker(params.snapshot, &params.project)?;
        let sig = setup.resolve_signature_handle(params.signature)?;
        let result = setup.checker.get_rest_type_of_signature(&sig);
        Ok(match result {
            None => None,
            Some(t) => setup.new_type_response(&t),
        })
    }

    pub fn handle_get_return_type_of_signature(
        &self,
        params: &GetSignaturePropertyParams,
    ) -> Result<Option<TypeResponse>, String> { ::tsox_core::fntrace::enter("handle_get_return_type_of_signature"); 
        let setup = self.setup_checker(params.snapshot, &params.project)?;
        let sig = setup.resolve_signature_handle(params.signature)?;
        let result = setup.checker.get_return_type_of_signature(&sig);
        Ok(match result {
            None => None,
            Some(t) => setup.new_type_response(&t),
        })
    }

    pub fn handle_get_semantic_diagnostics(
        &self,
        params: &GetDiagnosticsParams,
    ) -> Result<Vec<DiagnosticResponse>, String> { ::tsox_core::fntrace::enter("handle_get_semantic_diagnostics"); 
        self.get_diagnostics(params, |p, sf| {
            p.get_semantic_diagnostics(&mut core_context(), sf)
                .into_iter()
                .map(|d| (*d).clone())
                .collect()
        })
    }

    pub fn handle_get_shorthand_assignment_value_symbol(
        &self,
        params: &GetSymbolAtLocationParams,
    ) -> Result<Option<SymbolResponse>, String> { ::tsox_core::fntrace::enter("handle_get_shorthand_assignment_value_symbol"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let node = setup
            .sd
            .resolve_node_handle(&setup.program, &params.location)?;
        let Some(node) = node else { return Ok(None) };
        let symbol = setup
            .checker
            .get_shorthand_assignment_value_symbol(Some(&node));
        match symbol {
            None => Ok(None),
            Some(symbol) => Ok(setup.new_symbol_response(&symbol)),
        }
    }

    pub fn handle_get_signature_from_declaration(
        &self,
        params: &CheckerNodeParams,
    ) -> Result<Option<SignatureResponse>, String> { ::tsox_core::fntrace::enter("handle_get_signature_from_declaration"); 
        let mut setup = self.setup_checker(params.snapshot, &params.project)?;
        let node = setup
            .sd
            .resolve_node_handle(&setup.program, &params.location)?;
        let Some(node) = node else { return Ok(None) };
        let sig = setup.checker.get_signature_from_declaration(&node);
        Ok(match sig {
            None => None,
            Some(sig) => setup.new_signature_response(&sig),
        })
    }

    pub fn handle_get_signature_usages(
        &self,
        params: &GetSignatureUsagesParams,
    ) -> Result<Option<Vec<SignatureUsageResponse>>, String> { ::tsox_core::fntrace::enter("handle_get_signature_usages"); 
        let sd = self.get_snapshot_data(params.snapshot)?;
        let program = sd.get_program(&params.project)?;
        let signature_decl = sd.resolve_node_handle(&program, &params.signature_decl)?;
        let Some(signature_decl) = signature_decl else {
            return Ok(None);
        };
        let lang_svc = self.setup_language_service(&sd, &program, &params.project, "")?;
        let usages = lang_svc.get_signature_usages(&mut core_context(), &signature_decl);
        let usages = usages?;
        let Some(usages) = usages else {
            return Ok(None);
        };
        Ok(Some(
            usages
                .iter()
                .map(|u| SignatureUsageResponse {
                    name: sd.node_handle_from(&u.name),
                    call: u.call.as_ref().map(|c| sd.node_handle_from(c)),
                })
                .collect(),
        ))
    }

    pub fn handle_get_signatures_of_type(
        &self,
        params: &GetSignaturesOfTypeParams,
    ) -> Result<Vec<SignatureResponse>, String> { ::tsox_core::fntrace::enter("handle_get_signatures_of_type"); 
        let setup = self.setup_checker(params.snapshot, &params.project)?;
        let t = setup.resolve_type_handle(params.r#type)?;
        let sigs = setup
            .checker
            .get_signatures_of_type(
                &t,
                match params.kind {
                    1 => tsox_checker::checker::types::SignatureKind::Construct,
                    _ => tsox_checker::checker::types::SignatureKind::Call,
                },
            );
        Ok(sigs
            .iter()
            .filter_map(|sig| setup.new_signature_response(sig))
            .collect())
    }

    pub fn handle_get_source_file(
        &self,
        params: &GetSourceFileParams,
    ) -> Result<Option<SourceFileResponse>, String> { ::tsox_core::fntrace::enter("handle_get_source_file"); 
        let sd = self.get_snapshot_data(params.snapshot)?;
        let program = sd.get_program(&params.project)?;
        let source_file = program.get_source_file(&params.file.to_file_name());
        self.encode_source_file_response(source_file)
    }

    pub fn handle_get_source_file_metadata(
        &self,
        params: &GetSourceFileParams,
    ) -> Result<Option<SourceFileMetadata>, String> { ::tsox_core::fntrace::enter("handle_get_source_file_metadata"); 
        let sd = self.get_snapshot_data(params.snapshot)?;
        let program = sd.get_program(&params.project)?;
        let source_file = match program.get_source_file(&params.file.to_file_name()) {
            None => return Ok(None),
            Some(sf) => sf,
        };
        let meta_data = program.get_source_file_meta_data(&source_file.file_name);
        Ok(Some(SourceFileMetadata {
            is_default_library: program.is_source_file_default_library(&source_file.file_name),
            is_from_external_library: program.is_source_file_from_external_library(&source_file),
            package_json_type: meta_data.package_json_type.clone(),
            package_json_directory: meta_data.package_json_directory.clone(),
            implied_node_format: meta_data.implied_node_format as i32,
        }))
    }

    pub fn handle_get_source_file_names(
        &self,
        params: &GetSourceFileNamesParams,
    ) -> Result<Vec<String>, String> { ::tsox_core::fntrace::enter("handle_get_source_file_names"); 
        let sd = self.get_snapshot_data(params.snapshot)?;
        let program = sd.get_program(&params.project)?;
        Ok(program
            .get_source_files()
            .iter()
            .map(|sf| sf.file_name.clone())
            .collect())
    }

    pub fn handle_get_suggestion_diagnostics(
        &self,
        params: &GetDiagnosticsParams,
    ) -> Result<Vec<DiagnosticResponse>, String> { ::tsox_core::fntrace::enter("handle_get_suggestion_diagnostics"); 
        self.get_diagnostics(params, |p, sf| {
            p.get_suggestion_diagnostics(&mut core_context(), sf)
                .into_iter()
                .map(|d| (*d).clone())
                .collect()
        })
    }

    pub fn handle_get_symbol_at_location(
        &self,
        params: &GetSymbolAtLocationParams,
    ) -> Result<Option<SymbolResponse>, String> { ::tsox_core::fntrace::enter("handle_get_symbol_at_location"); 
        let setup = self.setup_checker(params.snapshot, &params.project)?;
        let node = setup
            .sd
            .resolve_node_handle(&setup.program, &params.location)?;
        let Some(node) = node else { return Ok(None) };
        let symbol = setup.checker.get_symbol_at_location(&node);
        match symbol {
            None => Ok(None),
            Some(symbol) => Ok(setup.new_symbol_response(&symbol)),
        }
    }

    pub fn handle_get_symbol_at_position(
        &self,
        params: &GetSymbolAtPositionParams,
    ) -> Result<Option<SymbolResponse>, String> { ::tsox_core::fntrace::enter("handle_get_symbol_at_position"); 
        let setup = self.setup_checker(params.snapshot, &params.project)?;
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
        let symbol = setup.checker.get_symbol_at_location(&node);
        match symbol {
            None => Ok(None),
            Some(symbol) => Ok(setup.new_symbol_response(&symbol)),
        }
    }

    pub fn handle_get_symbol_of_source_file(
        &self,
        params: &GetSymbolOfSourceFileParams,
    ) -> Result<Option<SymbolResponse>, String> { ::tsox_core::fntrace::enter("handle_get_symbol_of_source_file"); 
        let setup = self.setup_checker(params.snapshot, &params.project)?;
        let source_file = setup
            .program
            .get_source_file(&params.file.to_file_name())
            .ok_or_else(|| client_error(format!("source file not found: {}", params.file)))?;
        let symbol = setup.checker.get_symbol_at_location(&source_file.node);
        match symbol {
            None => Ok(None),
            Some(symbol) => Ok(setup.new_symbol_response(&symbol)),
        }
    }

    pub fn handle_get_symbol_of_type(
        &self,
        params: &GetTypePropertyParams,
    ) -> Result<Option<SymbolResponse>, String> { ::tsox_core::fntrace::enter("handle_get_symbol_of_type"); 
        self.resolve_symbol_property_of_type(params, |t| t.symbol().cloned())
    }

    pub fn handle_get_symbols_at_locations(
        &self,
        params: &GetSymbolsAtLocationsParams,
    ) -> Result<Vec<SymbolResponse>, String> { ::tsox_core::fntrace::enter("handle_get_symbols_at_locations"); 
        let setup = self.setup_checker(params.snapshot, &params.project)?;
        let mut results = Vec::with_capacity(params.locations.len());
        for loc in &params.locations {
            let node = setup.sd.resolve_node_handle(&setup.program, loc)?;
            if let Some(node) = node {
                if let Some(symbol) = setup.checker.get_symbol_at_location(&node) {
                    if let Some(resp) = setup.new_symbol_response(&symbol) {
                        results.push(resp);
                    }
                }
            }
        }
        Ok(results)
    }

    pub fn handle_get_symbols_at_positions(
        &self,
        params: &GetSymbolsAtPositionsParams,
    ) -> Result<Vec<SymbolResponse>, String> { ::tsox_core::fntrace::enter("handle_get_symbols_at_positions"); 
        let setup = self.setup_checker(params.snapshot, &params.project)?;
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
                if let Some(symbol) = setup.checker.get_symbol_at_location(&node) {
                    if let Some(resp) = setup.new_symbol_response(&symbol) {
                        results.push(resp);
                    }
                }
            }
        }
        Ok(results)
    }

    pub fn handle_get_symbols_in_scope(
        &self,
        params: &GetSymbolsInScopeParams,
    ) -> Result<Vec<SymbolResponse>, String> { ::tsox_core::fntrace::enter("handle_get_symbols_in_scope"); 
        let setup = self.setup_checker(params.snapshot, &params.project)?;
        let empty_handle = String::new();
        let location = setup.resolve_location(
            params.location.as_ref().unwrap_or(&empty_handle),
            params.file.as_ref(),
            params.position,
        )?;
        let Some(location) = location else {
            return Err(client_error("getSymbolsInScope requires a location"));
        };
        let symbols = setup.checker.get_symbols_in_scope(
            &location,
            tsox_frontend::ast::SymbolFlags::from_bits_retain(params.meaning),
        );
        Ok(symbols
            .iter()
            .filter_map(|symbol| setup.new_symbol_response(symbol))
            .collect())
    }

    pub fn handle_get_symbols_of_source_files(
        &self,
        params: &GetSymbolsOfSourceFilesParams,
    ) -> Result<Vec<SymbolResponse>, String> { ::tsox_core::fntrace::enter("handle_get_symbols_of_source_files"); 
        let setup = self.setup_checker(params.snapshot, &params.project)?;
        let mut results = Vec::with_capacity(params.files.len());
        for file in &params.files {
            let source_file = setup
                .program
                .get_source_file(&file.to_file_name())
                .ok_or_else(|| client_error(format!("source file not found: {}", file)))?;
            if let Some(symbol) = setup.checker.get_symbol_at_location(&source_file.node) {
                if let Some(resp) = setup.new_symbol_response(&symbol) {
                    results.push(resp);
                }
            }
        }
        Ok(results)
    }

    pub fn handle_get_syntactic_diagnostics(
        &self,
        params: &GetDiagnosticsParams,
    ) -> Result<Vec<DiagnosticResponse>, String> { ::tsox_core::fntrace::enter("handle_get_syntactic_diagnostics"); 
        self.get_diagnostics(params, |p, sf| {
            p.get_syntactic_diagnostics(&mut core_context(), sf)
                .into_iter()
                .map(|d| (*d).clone())
                .collect()
        })
    }

    pub fn handle_get_target_of_signature(
        &self,
        params: &GetSignaturePropertyParams,
    ) -> Result<Option<SignatureResponse>, String> { ::tsox_core::fntrace::enter("handle_get_target_of_signature"); 
        let sd = self.get_snapshot_data(params.snapshot)?;
        let sig = sd.resolve_signature_handle(&params.project, params.signature)?;
        let result = sig.target.clone();
        match result {
            None => Ok(None),
            Some(result) => Ok(sd.new_signature_response(&params.project, &result)),
        }
    }

    pub fn handle_get_target_of_type(
        &self,
        params: &GetTypePropertyParams,
    ) -> Result<Option<TypeResponse>, String> { ::tsox_core::fntrace::enter("handle_get_target_of_type"); 
        let sd = self.get_snapshot_data(params.snapshot)?;
        let t = sd.resolve_type_handle(&params.project, params.r#type)?;
        match t.target().cloned() {
            None => Ok(None),
            Some(result) => Ok(sd.new_type_response(&params.project, &result)),
        }
    }
}

use super::m5l::{
    core_context, new_diagnostic_responses, GetReferencedSymbolsForNodeParams,
    GetReferencesToSymbolInFileParams, GetSignatureUsagesParams, NodeHandle, SourceFileResponse,
};
