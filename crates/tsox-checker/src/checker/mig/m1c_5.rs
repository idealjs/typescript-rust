#![allow(unused_imports)]

#[allow(unused_imports, ambiguous_glob_reexports)]
use crate::checker::*;
#[allow(unused_imports)]
use tsox_frontend::ast::*;
#[allow(unused_imports)]
use tsox_core::diagnostics::messages_generated::*;

pub(crate) use crate::checker::checker::*;
pub(crate) use crate::checker::mig::m1c::*;
pub(crate) use crate::checker::mig::m1c_2::*;
pub(crate) use crate::checker::mig::m1c_3::*;
#[allow(unused_imports)]
use tsox_frontend::scanner::mig::m3i::declaration_name_to_string;
#[allow(unused_imports)]
use tsox_core::tspath::is_external_module_name_relative;
#[allow(unused_imports)]
use crate::checker::mig::m3a_2::{new_diagnostic_for_node, new_diagnostic_chain_for_node};
#[allow(unused_imports)]
use tsox_frontend::ast::mig::m3f_2::{node_parameters, node_type_parameters};
#[allow(unused_imports)]
use tsox_frontend::ast::mig::m3e::{RepopulateDiagnosticInfo, RepopulateDiagnosticKind};
#[allow(unused_imports)]
use tsox_tsoptions::module::ResolvedModule;
#[allow(unused_imports)]
use crate::checker::mig::m1c::r26k5_defs::{
    create_module_not_found_chain_details, create_mode_mismatch_details_worker,
};
#[allow(unused_imports)]
use tsox_core::core::compiler_options_kinds::ResolutionMode;
use std::sync::Arc;

use crate::checker::types::*;
use tsox_frontend::ast::{Diagnostic, Node, NodeData};

impl Checker {
    pub fn contextually_check_function_expression_or_object_literal_method(
        &mut self,
        node: &Arc<Node>,
        check_mode: CheckMode,
    ) {
        if self.node_links.get_or_default(node).flags.contains(NodeCheckFlags::ContextChecked) {
            return;
        }
        let contextual_signature = self.get_contextual_signature(node);
        if self.node_links.get_or_default(node).flags.contains(NodeCheckFlags::ContextChecked) {
            return;
        }
        self.node_links.get_or_default(node).flags = NodeCheckFlags::ContextChecked;;
        let node_symbol = self.get_symbol_of_declaration(node);
        let function_type = self.get_type_of_symbol(node_symbol.as_ref().unwrap());
        let signature = self
            .get_signatures_of_type(&function_type, SignatureKind::Call)
            .first()
            .cloned();
        let mut signature = match signature {
            Some(s) => s,
            None => return,
        };
        if self.is_context_sensitive(node) {
            if let Some(contextual_signature) = &contextual_signature {
                let inference_context = self.get_inference_context_arc(node).cloned();
                let mut instantiated_contextual_signature: Option<Arc<Signature>> = None;
                if check_mode.contains(CheckMode::Inferential) {
                    if let Some(inference_context) = &inference_context {
                        self.infer_from_annotated_parameters_and_return(
                            &signature,
                            contextual_signature,
                            inference_context,
                        );
                        let rest_type = self.get_effective_rest_type(contextual_signature);
                        if let Some(rest_type) = rest_type {
                            if rest_type.flags.contains(TypeFlags::TypeParameter) {
                                instantiated_contextual_signature = Some(
                                    self.instantiate_signature(
                                        contextual_signature,
                                        inference_context.non_fixing_mapper.as_ref(),
                                    ),
                                );
                            }
                        }
                    }
                }
                if instantiated_contextual_signature.is_none() {
                    instantiated_contextual_signature = Some(match &inference_context {
                        Some(inference_context) => self.instantiate_signature(
                            contextual_signature,
                            inference_context.mapper.as_ref(),
                        ),
                        None => contextual_signature.clone(),
                    });
                }
                self.assign_contextual_parameter_types(
                    Arc::make_mut(&mut signature),
                    instantiated_contextual_signature.as_ref().unwrap(),
                );
            } else {
                self.assign_non_contextual_parameter_types(&signature);
            }
        } else if contextual_signature.is_some()
            && node_type_parameters(node).map(|l| l.is_empty()).unwrap_or(true)
            && contextual_signature.as_ref().unwrap().parameters.len()
                > node_parameters(node).map(|l| l.nodes.len()).unwrap_or(0)
        {
            let inference_context = self.get_inference_context_arc(node).cloned();
            if check_mode.contains(CheckMode::Inferential) {
                if let (Some(inference_context), Some(contextual_signature)) =
                    (&inference_context, &contextual_signature)
                {
                    self.infer_from_annotated_parameters_and_return(
                        &signature,
                        contextual_signature,
                        inference_context,
                    );
                }
            }
        }
        if contextual_signature.is_some()
            && signature.resolved_return_type.get().is_none()
        {
            let return_type = self.get_return_type_from_body(node, check_mode);
            if signature.resolved_return_type.get().is_none() {
                signature.resolved_return_type.set(return_type).ok();
            }
        }
        self.check_signature_declaration(node);
    }

    pub fn combine_symbol_tables(
        &mut self,
        first: &SymbolTable,
        second: &SymbolTable,
    ) -> SymbolTable {
        if first.is_empty() {
            return second.clone();
        }
        if second.is_empty() {
            return first.clone();
        }
        let mut combined = SymbolTable::new();
        self.merge_symbol_table(&mut combined, first, false, None);
        self.merge_symbol_table(&mut combined, second, false, None);
        combined
    }

    pub fn combine_value_and_type_symbols(
        &mut self,
        value_symbol: &Arc<Symbol>,
        type_symbol: &Arc<Symbol>,
    ) -> Arc<Symbol> {
        if value_symbol.id() == self.unknown_symbol().id() && type_symbol.id() == self.unknown_symbol().id() {
            return self.unknown_symbol();
        }
        if type_symbol.flags.contains(SymbolFlags::VALUE) {
            return type_symbol.clone();
        }
        if value_symbol
            .flags
            .intersects(SymbolFlags::TYPE | SymbolFlags::NAMESPACE)
        {
            return value_symbol.clone();
        }
        let mut result = self.new_symbol(value_symbol.flags | type_symbol.flags, &value_symbol.name);
        let mut declarations = value_symbol.declarations.clone();
        declarations.extend(type_symbol.declarations.iter().cloned());
        declarations.dedup_by(|a, b| Arc::ptr_eq(a, b));
        if let Some(result_mut) = Arc::get_mut(&mut result) {
            result_mut.declarations = declarations;
            result_mut.value_declaration = value_symbol.value_declaration.clone();
            result_mut.members = type_symbol.members.clone();
            result_mut.exports = value_symbol.exports.clone();
        }
        if let Some(parent) = value_symbol.parent().or_else(|| type_symbol.parent()) {
            result.set_parent(&parent);
        }
        result
    }

    pub fn error_no_module_member_symbol(
        &mut self,
        module_symbol: &Arc<Symbol>,
        target_symbol: Option<&Arc<Symbol>>,
        node: &Arc<Node>,
        name: &Arc<Node>,
    ) {
        if self.compiler_options.no_check.is_true() {
            return;
        }
        let module_name = self.get_fully_qualified_name(module_symbol, Some(node));
        let declaration_name = declaration_name_to_string(Some(name));
        let suggestion = if is_identifier(name) {
            target_symbol
                .and_then(|ts| self.get_suggested_symbol_for_nonexistent_module(name, ts))
        } else {
            None
        };
        if let Some(suggestion) = suggestion {
            let suggestion_name = self.symbol_to_string(&suggestion);
            if let Some(mut diagnostic) = self.error_message(
                name,X_0_HAS_NO_EXPORTED_MEMBER_NAMED_1_DID_YOU_MEAN_2,
                &[module_name, declaration_name.clone(), suggestion_name.clone()],
            ) {
                if let Some(value_declaration) = &suggestion.value_declaration {
                    let related = create_diagnostic_for_node_message(
                        value_declaration,X_0_IS_DECLARED_HERE,
                        &[suggestion_name],
                    );
                    Arc::make_mut(&mut diagnostic).add_related_info((*related).clone());
                }
            }
        } else if module_symbol
            .exports
            .get(INTERNAL_SYMBOL_NAME_DEFAULT)
            .is_some()
        {
            self.error_message(
                name,MODULE_0_HAS_NO_EXPORTED_MEMBER_1_DID_YOU_MEAN_TO_USE_IMPORT_1_FROM_0_INSTEAD,
                &[module_name, declaration_name],
            );
        } else {
            self.report_non_exported_member(name, &declaration_name, module_symbol, &module_name);
        }
    }

    pub fn error_on_implicit_any_module(
        &mut self,
        is_error: bool,
        error_node: &Arc<Node>,
        mode: ResolutionMode,
        resolved_module: &ResolvedModule,
        module_reference: &str,
    ) {
        if is_side_effect_import(error_node) {
            return;
        }
        let mut error_info: Option<Arc<Diagnostic>> = None;
        let package_id_name = resolved_module
            .package_id
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_default();
        if !is_external_module_name_relative(module_reference) && !package_id_name.is_empty() {
            error_info = Some(self.create_module_not_found_chain(
                resolved_module,
                error_node,
                module_reference,
                mode,
                &package_id_name,
            ));
        }
        let diagnostic = new_diagnostic_chain_for_node(
            error_info.as_deref(),
            Some(error_node),
            COULD_NOT_FIND_A_DECLARATION_FILE_FOR_MODULE_0_1_IMPLICITLY_HAS_AN_ANY_TYPE,
            vec![module_reference.to_string(), resolved_module.resolved_file_name.clone()],
        );
        self.add_error_or_suggestion(is_error, diagnostic);
    }

    pub fn create_module_not_found_chain(
        &mut self,
        resolved_module: &ResolvedModule,
        error_node: &Arc<Node>,
        module_reference: &str,
        mode: ResolutionMode,
        package_name: &str,
    ) -> Arc<Diagnostic> {
        let mut stored_package_name = package_name.to_string();
        if stored_package_name == module_reference {
            stored_package_name = String::new();
        }
        let source_file = self.get_source_file_of_node(error_node);
        let details = create_module_not_found_chain_details(
            self.program.as_ref(),
            source_file.as_ref(),
            module_reference,
            mode,
            package_name,
        );
        let mut result = new_diagnostic_for_node(Some(error_node), details.message, details.args);
        result.set_repopulate_info(Some(RepopulateDiagnosticInfo {
            kind: RepopulateDiagnosticKind::ModuleNotFound,
            module_reference: module_reference.to_string(),
            mode,
            package_name: stored_package_name,
        }));
        Arc::new(result)
    }

    pub fn create_mode_mismatch_details(
        &mut self,
        source_file: &Arc<SourceFile>,
        error_node: &Arc<Node>,
    ) -> Arc<Diagnostic> {
        let details = create_mode_mismatch_details_worker(self.program.as_ref(), source_file);
        let mut result = new_diagnostic_for_node(Some(error_node), details.message, details.args);
        result.set_repopulate_info(Some(RepopulateDiagnosticInfo {
            kind: RepopulateDiagnosticKind::ModeMismatch,
            module_reference: String::new(),
            mode: ResolutionMode::default(),
            package_name: String::new(),
        }));
        Arc::new(result)
    }

}

