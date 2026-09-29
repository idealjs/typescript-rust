#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_checker::checker::Checker;
use tsox_checker::checker::Signature;
use tsox_frontend::ast::{self, Node, SourceFile, Symbol};

use super::m5v2::M5v2SignatureInformation;
use super::m5w::M5wArgumentListInfo;
use super::m5w::M5wSignatureHelpItemInfo;
use crate::ls::language_service::LanguageService;

mod lsproto {
    pub use crate::lsp::lsproto::*;

    pub use crate::mig::m5m::get_client_capabilities;
    pub use crate::mig::m5m::ResolvedClientCapabilitiesContext as Context;
}

fn signature_help_documentation_format(ctx: &lsproto::Context) -> lsproto::MarkupKind {
    let formats: Vec<lsproto::MarkupKind> = lsproto::get_client_capabilities(ctx)
        .raw
        .pointer("/textDocument/signatureHelp/signatureInformation/documentationFormat")
        .and_then(|v| v.as_array())
        .map(|array| {
            array
                .iter()
                .filter_map(|v| match v.as_str()? {
                    "markdown" => Some(lsproto::MarkupKind::Markdown),
                    "plaintext" => Some(lsproto::MarkupKind::PlainText),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();
    lsproto::preferred_markup_kind(&formats)
}

fn signature_help_capability_pointer(ctx: &lsproto::Context, leaf: &str) -> bool {
    lsproto::get_client_capabilities(ctx)
        .raw
        .pointer(&format!("/textDocument/signatureHelp/signatureInformation/{leaf}"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn vs_supports_visual_studio_extensions(ctx: &lsproto::Context) -> bool {
    lsproto::get_client_capabilities(ctx)
        .raw
        .pointer("/_vs_supportsVisualStudioExtensions")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

impl LanguageService {
    pub fn create_signature_help_items(
        &self,
        ctx: &lsproto::Context,
        candidates: &[Arc<Signature>],
        resolved_signature: &Arc<Signature>,
        argument_info: &M5wArgumentListInfo,
        source_file: &Arc<SourceFile>,
        c: &mut Checker,
        use_full_prefix: bool,
    ) -> Option<crate::ls::types::SignatureHelp> {
        let doc_format = signature_help_documentation_format(ctx);
        let vs_capability = vs_supports_visual_studio_extensions(ctx);

        let enclosing_declaration =
            super::m5w::get_enclosing_declaration_from_invocation(&argument_info.invocation)?;

        let call_target_symbol: Option<Arc<Symbol>> =
            if let Some(contextual) = &argument_info.invocation.contextual_invocation {
                contextual.symbol.clone()
            } else {
                let mut target = super::m5w::get_expression_from_invocation(argument_info)
                    .and_then(|expression| c.get_symbol_at_location(&expression));
                if target.is_none() && use_full_prefix {
                    if let Some(declaration) = resolved_signature.declaration.as_ref() {
                        target = c.get_symbol_of_node(declaration);
                    }
                }
                target
            };

        let mut call_target_display_parts = String::new();
        if let Some(call_target) = &call_target_symbol {
            if !call_target.name.starts_with(ast::INTERNAL_SYMBOL_NAME_PREFIX) {
                if use_full_prefix {
                    call_target_display_parts.push_str(&c.symbol_to_string_ex(
                        call_target,
                        tsox_checker::checker::SymbolFormatFlags::UseAliasDefinedOutsideCurrentScope,
                        ast::SymbolFlags::None,
                    ));
                } else {
                    call_target_display_parts.push_str(&c.symbol_to_string(call_target));
                }
            }
        }

        let mut items: Vec<Vec<M5v2SignatureInformation>> = Vec::with_capacity(candidates.len());
        for candidate in candidates {
            items.push(self.m5v2_get_signature_help_item(
                candidate,
                argument_info.is_type_parameter_list,
                &call_target_display_parts,
                call_target_symbol.as_ref(),
                &enclosing_declaration,
                source_file,
                c,
                &doc_format,
                vs_capability,
            ));
        }

        let mut selected_item_index = 0usize;
        let mut item_seen = 0usize;
        for (i, item) in items.iter().enumerate() {
            if candidates[i].id == resolved_signature.id {
                selected_item_index = item_seen;
                if item.len() > 1 {
                    let mut count = 0usize;
                    for j in item {
                        if j.is_variadic || j.parameters.len() >= argument_info.argument_count {
                            selected_item_index = item_seen + count;
                            break;
                        }
                        count += 1;
                    }
                }
            }
            item_seen += item.len();
        }

        let mut flattened_signatures: Vec<M5v2SignatureInformation> = Vec::new();
        for item in items {
            flattened_signatures.extend(item);
        }
        if flattened_signatures.is_empty() {
            return None;
        }

        let sig_info_caps_active_parameter_support =
            signature_help_capability_pointer(ctx, "activeParameterSupport");
        let supports_per_signature_active_param = sig_info_caps_active_parameter_support;
        let supports_null_active_param =
            signature_help_capability_pointer(ctx, "noActiveParameterSupport");

        let mut signature_information: Vec<crate::ls::types::SignatureInformation> =
            Vec::with_capacity(flattened_signatures.len());
        for item in &flattened_signatures {
            let parameters: Vec<crate::ls::types::ParameterInformation> = item
                .parameters
                .iter()
                .map(|param| param.parameter_info.clone())
                .collect();
            let sig_info = crate::ls::types::SignatureInformation {
                label: item.label.clone(),
                documentation: item.documentation.clone(),
                parameters,
            };
            signature_information.push(sig_info);
        }

        let mut help = crate::ls::types::SignatureHelp {
            signatures: signature_information,
            active_signature: Some(selected_item_index as u32),
            active_parameter: None,
        };
        if !supports_per_signature_active_param {
            help.active_parameter = self.compute_active_parameter(
                &flattened_signatures[selected_item_index],
                argument_info.argument_index,
                supports_null_active_param,
            );
        }
        Some(help)
    }

    pub fn m5v2_get_signature_help_item(
        &self,
        candidate: &Arc<Signature>,
        is_type_parameter_list: bool,
        call_target_symbol: &str,
        call_target_sym: Option<&Arc<Symbol>>,
        enclosing_declaration: &Arc<Node>,
        source_file: &Arc<SourceFile>,
        c: &mut Checker,
        doc_format: &crate::lsp::lsproto_lsp_basic::MarkupKind,
        vs_capability: bool,
    ) -> Vec<M5v2SignatureInformation> {
        let infos: Vec<M5wSignatureHelpItemInfo> = if is_type_parameter_list {
            self.item_info_for_type_parameters(
                candidate,
                c,
                enclosing_declaration,
                source_file,
                doc_format,
                vs_capability,
            )
        } else {
            self.item_info_for_parameters(
                candidate,
                c,
                enclosing_declaration,
                source_file,
                doc_format,
                vs_capability,
            )
        };

        let suffix_dpw = super::m5w_3::return_type_to_display_parts(
            candidate,
            c,
            enclosing_declaration,
            source_file,
            vs_capability,
        );

        let mut documentation: Option<String> = None;
        if let Some(declaration) = candidate.declaration.as_ref() {
            let mapper = self
                .documentation_location_mapper(crate::ls::mig::m5s::SpanFeature::Definition);
            let doc = super::m5t2_2::get_documentation_from_declaration(
                &mapper,
                c,
                None,
                Some(declaration),
                declaration,
                super::m5w_3::markup_kind_str(doc_format),
                true,
            );
            if !doc.is_empty() {
                documentation = Some(doc);
            }
        }

        infos
            .into_iter()
            .map(|info| {
                let mut label_dpw =
                    crate::ls::display_parts_writer::new_display_parts_writer(vs_capability);
                if !call_target_symbol.is_empty() {
                    if let Some(sym) = call_target_sym {
                        label_dpw.write_symbol(call_target_symbol, sym);
                    }
                }
                label_dpw.write_from(&info.writer);
                label_dpw.write_from(&suffix_dpw);

                M5v2SignatureInformation {
                    label: label_dpw.as_string().to_string(),
                    documentation: documentation.clone(),
                    parameters: info.parameters,
                    is_variadic: info.is_variadic,
                    colorized_runs: label_dpw.get_runs().to_vec(),
                }
            })
            .collect()
    }
}
