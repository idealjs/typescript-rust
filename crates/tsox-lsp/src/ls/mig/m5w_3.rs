#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_checker::checker::Checker;
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::{self, Node, NodeData, SourceFile, Symbol, SyntaxKind};

use super::m5w::{
    M5wArgumentListInfo, M5wSignatureHelpItemInfo, M5wSignatureHelpParameter,
    SIGNATURE_HELP_NODE_BUILDER_FLAGS,
};
use crate::ls::types_symbols::{ParameterInformation, SignatureInformation};

type LspPrinter = tsox_frontend::format::mig::m4o_2::Printer;

fn signature_help_printer_options() -> tsox_frontend::format::mig::m4o_2::PrinterOptions { ::tsox_core::fntrace::enter("signature_help_printer_options"); 
    tsox_frontend::format::mig::m4o_2::PrinterOptions {
        remove_comments: false,
        new_line: tsox_core::core::compiler_options_kinds::NewLineKind::LF,
        omit_trailing_semicolon: false,
        no_emit_helpers: false,
        target: Default::default(),
        source_map: false,
        inline_source_map: false,
        inline_sources: false,
        omit_brace_source_map_positions: false,
        only_print_jsdoc_style: false,
        never_ascii_escape: false,
        preserve_source_newlines: false,
        terminate_unterminated_literals: false,
    }
}

fn new_signature_help_printer() -> LspPrinter { ::tsox_core::fntrace::enter("new_signature_help_printer"); 
    tsox_frontend::format::mig::m4o_2::new_printer(
        signature_help_printer_options(),
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
        tsox_frontend::format::mig::m4o_2::EmitContext,
    )
}

fn signature_help_node_builder_flags() -> tsox_checker::checker::symboltracker::NodeBuilderFlags { ::tsox_core::fntrace::enter("signature_help_node_builder_flags"); 
    use tsox_checker::checker::symboltracker::NodeBuilderFlags as Flags;
    Flags::OmitParameterModifiers | Flags::UseAliasDefinedOutsideCurrentScope
}

pub fn markup_kind_str(doc_format: &crate::lsp::lsproto_lsp_basic::MarkupKind) -> &'static str { ::tsox_core::fntrace::enter("markup_kind_str"); 
    match doc_format {
        crate::lsp::lsproto_lsp_basic::MarkupKind::Markdown => "markdown",
        _ => "plaintext",
    }
}

fn symbol_documentation(
    ls: &crate::ls::language_service::LanguageService,
    parameter: &Arc<Symbol>,
    c: &mut Checker,
    doc_format: &crate::lsp::lsproto_lsp_basic::MarkupKind,
) -> Option<String> { ::tsox_core::fntrace::enter("symbol_documentation"); 
    let declaration = parameter.value_declaration.as_ref()?;
    let mapper = ls
        .documentation_location_mapper(crate::ls::mig::m5s::SpanFeature::Definition);
    let doc = super::m5t2_2::get_documentation_from_declaration(
        &mapper,
        c,
        None,
        Some(declaration),
        declaration,
        markup_kind_str(doc_format),
        true,
    );
    if doc.is_empty() {
        None
    } else {
        Some(doc)
    }
}

impl crate::ls::language_service::LanguageService {
    pub fn get_signature_help_item(
        &self,
        candidate: &Arc<tsox_checker::checker::Signature>,
        is_type_parameter_list: bool,
        call_target_symbol: &str,
        call_target_sym: Option<&Arc<Symbol>>,
        enclosing_declaration: &Arc<Node>,
        source_file: &Arc<SourceFile>,
        c: &mut Checker,
        doc_format: &crate::lsp::lsproto_lsp_basic::MarkupKind,
        vs_capability: bool,
    ) -> Vec<SignatureInformation> { ::tsox_core::fntrace::enter("get_signature_help_item"); 
        let infos = if is_type_parameter_list {
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

        let suffix_dpw = return_type_to_display_parts(
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
                Some(&declaration),
                &declaration,
                markup_kind_str(doc_format),
                true,
            );
            if !doc.is_empty() {
                documentation = Some(doc);
            }
        }

        infos
            .iter()
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

                SignatureInformation {
                    label: label_dpw.as_string().to_string(),
                    documentation: documentation.clone(),
                    parameters: info
                        .parameters
                        .iter()
                        .map(|p| p.parameter_info.clone())
                        .collect(),
                }
            })
            .collect()
    }

    pub fn item_info_for_type_parameters(
        &self,
        candidate_signature: &Arc<tsox_checker::checker::Signature>,
        c: &mut Checker,
        enclosing_declaration: &Arc<Node>,
        source_file: &Arc<SourceFile>,
        doc_format: &crate::lsp::lsproto_lsp_basic::MarkupKind,
        vs_capability: bool,
    ) -> Vec<M5wSignatureHelpItemInfo> { ::tsox_core::fntrace::enter("item_info_for_type_parameters"); 
        let mut p = new_signature_help_printer();

        let type_parameters: &[Arc<tsox_checker::checker::Type>] = if let Some(target) =
            candidate_signature.target.as_ref()
        {
            &target.type_parameters
        } else {
            &candidate_signature.type_parameters
        };
        let signature_help_type_parameters: Vec<M5wSignatureHelpParameter> = type_parameters
            .iter()
            .filter_map(|type_parameter| {
                create_signature_help_parameter_for_type_parameter(
                    type_parameter,
                    source_file,
                    enclosing_declaration,
                    c,
                    &mut p,
                )
            })
            .collect();

        let mut this_parameter: Vec<M5wSignatureHelpParameter> = Vec::new();
        if let Some(this_param) = candidate_signature.this_parameter() {
            this_parameter.push(self.m5w_create_signature_help_parameter_for_parameter(
                &this_param,
                enclosing_declaration,
                &mut p,
                source_file,
                c,
                doc_format,
            ));
        }

        let mut dpw = crate::ls::display_parts_writer::new_display_parts_writer(vs_capability);
        dpw.write_punctuation(tsox_frontend::scanner::token_to_string(
            SyntaxKind::LessThanToken,
        ));
        for (i, type_parameter) in signature_help_type_parameters.iter().enumerate() {
            if i > 0 {
                dpw.write_punctuation(", ");
            }
            let label = type_parameter.parameter_info.label.clone();
            dpw.write_classified(&label, "typeParameterName");
        }
        dpw.write_punctuation(tsox_frontend::scanner::token_to_string(
            SyntaxKind::GreaterThanToken,
        ));

        let lists = c.get_expanded_parameters(candidate_signature, false);
        if !lists.is_empty() {
            dpw.write_punctuation(tsox_frontend::scanner::token_to_string(
                SyntaxKind::OpenParenToken,
            ));
        }

        let mut result = Vec::with_capacity(lists.len());
        for parameter_list in &lists {
            let mut param_dpw =
                crate::ls::display_parts_writer::new_display_parts_writer(vs_capability);
            param_dpw.write_from(&dpw);

            let mut parameters = this_parameter.clone();
            for (j, param) in parameter_list.iter().enumerate() {
                let param_node = {
                    let mut node_builder = tsox_checker::checker::mig::m2f::new_node_builder_ex(
                        c,
                        tsox_checker::checker::mig::m2f::new_emit_context(),
                        std::collections::HashMap::new(),
                    );
                    node_builder.symbol_to_parameter_declaration(
                        param,
                        Some(enclosing_declaration),
                        signature_help_node_builder_flags(),
                        tsox_checker::checker::symboltracker::NodeBuilderInternalFlags::default(),
                        None,
                    )
                };
                let Some(param_node) = param_node else {
                    continue;
                };

                if j > 0 {
                    param_dpw.write_punctuation(", ");
                }
                let param_label = p.emit(&param_node, Some(source_file));
                param_dpw.write(&param_label);

                let parameter = self.m5w_create_signature_help_parameter_from_label(
                    param,
                    &param_label,
                    c,
                    doc_format,
                );
                parameters.push(parameter);
            }
            param_dpw.write_punctuation(tsox_frontend::scanner::token_to_string(
                SyntaxKind::CloseParenToken,
            ));

            result.push(M5wSignatureHelpItemInfo {
                is_variadic: false,
                parameters: signature_help_type_parameters.clone(),
                writer: param_dpw,
            });
        }
        result
    }

    pub fn item_info_for_parameters(
        &self,
        candidate_signature: &Arc<tsox_checker::checker::Signature>,
        c: &mut Checker,
        enclosing_declaration: &Arc<Node>,
        source_file: &Arc<SourceFile>,
        doc_format: &crate::lsp::lsproto_lsp_basic::MarkupKind,
        vs_capability: bool,
    ) -> Vec<M5wSignatureHelpItemInfo> { ::tsox_core::fntrace::enter("item_info_for_parameters"); 
        let mut p = new_signature_help_printer();

        let type_parameters = &candidate_signature.type_parameters;
        let mut signature_help_type_parameters: Vec<M5wSignatureHelpParameter> =
            Vec::with_capacity(type_parameters.len());
        for type_parameter in type_parameters {
            if let Some(parameter) = create_signature_help_parameter_for_type_parameter(
                type_parameter,
                source_file,
                enclosing_declaration,
                c,
                &mut p,
            ) {
                signature_help_type_parameters.push(parameter);
            }
        }

        let mut dpw = crate::ls::display_parts_writer::new_display_parts_writer(vs_capability);
        if !signature_help_type_parameters.is_empty() {
            dpw.write_punctuation(tsox_frontend::scanner::token_to_string(
                SyntaxKind::LessThanToken,
            ));
            for (i, type_parameter) in signature_help_type_parameters.iter().enumerate() {
                if i > 0 {
                    dpw.write_punctuation(", ");
                }
                let label = type_parameter.parameter_info.label.clone();
                dpw.write_classified(&label, "typeParameterName");
            }
            dpw.write_punctuation(tsox_frontend::scanner::token_to_string(
                SyntaxKind::GreaterThanToken,
            ));
        }

        let lists = c.get_expanded_parameters(candidate_signature, false);
        if !lists.is_empty() {
            dpw.write_punctuation(tsox_frontend::scanner::token_to_string(
                SyntaxKind::OpenParenToken,
            ));
        }

        let has_effective_rest = c.has_effective_rest_parameter(candidate_signature);

        let mut result = Vec::with_capacity(lists.len());
        for parameter_list in &lists {
            let mut parameters: Vec<M5wSignatureHelpParameter> =
                Vec::with_capacity(parameter_list.len());
            let mut param_dpw =
                crate::ls::display_parts_writer::new_display_parts_writer(vs_capability);
            param_dpw.write_from(&dpw);

            for (j, param) in parameter_list.iter().enumerate() {
                let param_node = {
                    let mut node_builder = tsox_checker::checker::mig::m2f::new_node_builder_ex(
                        c,
                        tsox_checker::checker::mig::m2f::new_emit_context(),
                        std::collections::HashMap::new(),
                    );
                    node_builder.symbol_to_parameter_declaration(
                        param,
                        Some(enclosing_declaration),
                        signature_help_node_builder_flags(),
                        tsox_checker::checker::symboltracker::NodeBuilderInternalFlags::default(),
                        None,
                    )
                };
                let Some(param_node) = param_node else {
                    continue;
                };

                if j > 0 {
                    param_dpw.write_punctuation(", ");
                }
                let param_label = p.emit(&param_node, Some(source_file));
                param_dpw.write(&param_label);

                let parameter = self.m5w_create_signature_help_parameter_from_label(
                    param,
                    &param_label,
                    c,
                    doc_format,
                );
                parameters.push(parameter);
            }
            param_dpw.write_punctuation(tsox_frontend::scanner::token_to_string(
                SyntaxKind::CloseParenToken,
            ));

            let is_variadic = has_effective_rest
                && (lists.len() == 1
                    || parameter_list
                        .last()
                        .map(|last| {
                            last.check_flags
                                .contains(tsox_frontend::ast::CheckFlags::RestParameter)
                        })
                        .unwrap_or(false));

            result.push(M5wSignatureHelpItemInfo {
                is_variadic,
                parameters,
                writer: param_dpw,
            });
        }
        result
    }

    pub fn m5w_create_signature_help_parameter_from_label(
        &self,
        parameter: &Arc<Symbol>,
        label: &str,
        c: &mut Checker,
        doc_format: &crate::lsp::lsproto_lsp_basic::MarkupKind,
    ) -> M5wSignatureHelpParameter { ::tsox_core::fntrace::enter("m5w_create_signature_help_parameter_from_label"); 
        let parameter_info = ParameterInformation {
            label: label.to_string(),
            documentation: symbol_documentation(self, parameter, c, doc_format),
        };
        M5wSignatureHelpParameter {
            parameter_info,
            is_variadic: parameter
                .check_flags
                .contains(tsox_frontend::ast::CheckFlags::RestParameter),
        }
    }

    pub fn m5w_create_signature_help_parameter_for_parameter(
        &self,
        parameter: &Arc<Symbol>,
        enclosing_declaration: &Arc<Node>,
        p: &mut LspPrinter,
        source_file: &Arc<SourceFile>,
        c: &mut Checker,
        doc_format: &crate::lsp::lsproto_lsp_basic::MarkupKind,
    ) -> M5wSignatureHelpParameter { ::tsox_core::fntrace::enter("m5w_create_signature_help_parameter_for_parameter"); 
        let label =
            get_parameter_label_string(parameter, source_file, enclosing_declaration, p, c);
        self.m5w_create_signature_help_parameter_from_label(parameter, &label, c, doc_format)
    }
}

pub fn create_signature_help_parameter_for_type_parameter(
    t: &Arc<tsox_checker::checker::Type>,
    source_file: &Arc<SourceFile>,
    enclosing_declaration: &Arc<Node>,
    c: &mut Checker,
    p: &mut LspPrinter,
) -> Option<M5wSignatureHelpParameter> { ::tsox_core::fntrace::enter("create_signature_help_parameter_for_type_parameter"); 
    let type_parameter_node = {
        let mut node_builder = tsox_checker::checker::mig::m2f::new_node_builder_ex(
            c,
            tsox_checker::checker::mig::m2f::new_emit_context(),
            std::collections::HashMap::new(),
        );
        node_builder.type_parameter_to_declaration(
            t,
            Some(enclosing_declaration),
            signature_help_node_builder_flags(),
            tsox_checker::checker::symboltracker::NodeBuilderInternalFlags::default(),
            None,
        )
    };
    let type_parameter_node = type_parameter_node?;
    let label = p.emit(&type_parameter_node, Some(source_file));
    let parameter_info = ParameterInformation {
        label,
        documentation: None,
    };
    Some(M5wSignatureHelpParameter {
        parameter_info,
        is_variadic: false,
    })
}

pub fn get_parameter_label_string(
    parameter: &Arc<Symbol>,
    source_file: &Arc<SourceFile>,
    enclosing_declaration: &Arc<Node>,
    p: &mut LspPrinter,
    c: &mut Checker,
) -> String { ::tsox_core::fntrace::enter("get_parameter_label_string"); 
    let param_node = {
        let mut node_builder = tsox_checker::checker::mig::m2f::new_node_builder_ex(
            c,
            tsox_checker::checker::mig::m2f::new_emit_context(),
            std::collections::HashMap::new(),
        );
        node_builder.symbol_to_parameter_declaration(
            parameter,
            Some(enclosing_declaration),
            signature_help_node_builder_flags(),
            tsox_checker::checker::symboltracker::NodeBuilderInternalFlags::default(),
            None,
        )
    };
    match param_node {
        Some(param_node) => p.emit(&param_node, Some(source_file)),
        None => String::new(),
    }
}

pub fn return_type_to_display_parts(
    candidate_signature: &Arc<tsox_checker::checker::Signature>,
    c: &mut Checker,
    enclosing_declaration: &Arc<Node>,
    source_file: &Arc<SourceFile>,
    vs_capability: bool,
) -> crate::ls::display_parts_writer::DisplayPartsWriter { ::tsox_core::fntrace::enter("return_type_to_display_parts"); 
    let mut dpw = crate::ls::display_parts_writer::new_display_parts_writer(vs_capability);
    dpw.write_punctuation(": ");

    if let Some(predicate) = c.get_type_predicate_of_signature(candidate_signature) {
        dpw.write(&c.type_predicate_to_string(&predicate));
    } else if let Some(return_type) = c.get_return_type_of_signature(candidate_signature) {
        let type_node = c.type_to_type_node(&return_type);
        let mut p = new_signature_help_printer();
        let text = p.emit(&type_node, Some(source_file));
        dpw.write(&text);
    }
    dpw
}

pub fn get_type_help_item(
    symbol: &Arc<Symbol>,
    type_parameters: &[Arc<tsox_checker::checker::Type>],
    enclosing_declaration: &Arc<Node>,
    source_file: &Arc<SourceFile>,
    c: &mut Checker,
) -> SignatureInformation { ::tsox_core::fntrace::enter("get_type_help_item"); 
    let mut p = new_signature_help_printer();

    let parameters: Vec<M5wSignatureHelpParameter> = type_parameters
        .iter()
        .filter_map(|type_param| {
            create_signature_help_parameter_for_type_parameter(
                type_param,
                source_file,
                enclosing_declaration,
                c,
                &mut p,
            )
        })
        .collect();

    let mut display_parts = String::new();
    display_parts.push_str(&c.symbol_to_string(symbol));
    if !parameters.is_empty() {
        display_parts.push_str(tsox_frontend::scanner::token_to_string(
            SyntaxKind::LessThanToken,
        ));
        for (i, type_parameter) in parameters.iter().enumerate() {
            if i > 0 {
                display_parts.push_str(", ");
            }
            display_parts.push_str(&type_parameter.parameter_info.label.clone());
        }
        display_parts.push_str(tsox_frontend::scanner::token_to_string(
            SyntaxKind::GreaterThanToken,
        ));
    }

    SignatureInformation {
        label: display_parts,
        documentation: None,
        parameters: parameters
            .iter()
            .map(|param| param.parameter_info.clone())
            .collect(),
    }
}
