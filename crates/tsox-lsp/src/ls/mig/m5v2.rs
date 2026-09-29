#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_checker::checker::Checker;
use tsox_checker::checker::Signature;
use tsox_frontend::ast::{self, Node, SourceFile, Symbol};

use super::m5w::M5wArgumentListInfo;
use crate::ls::language_service::LanguageService;
use crate::ls::display_parts_writer::VsClassifiedTextRun;
use crate::ls::types::SignatureHelp;

mod lsproto {
    pub use crate::lsp::lsproto::*;

    pub use crate::mig::m5m::get_client_capabilities;
    pub use crate::mig::m5m::ResolvedClientCapabilitiesContext as Context;
}

pub struct M5v2SignatureInformation {
    pub label: String,
    pub documentation: Option<String>,
    pub parameters: Vec<super::m5w::M5wSignatureHelpParameter>,
    pub is_variadic: bool,
    pub colorized_runs: Vec<VsClassifiedTextRun>,
}

impl LanguageService {
    pub fn create_js_signature_help_items(
        &self,
        ctx: &lsproto::Context,
        argument_info: &M5wArgumentListInfo,
        program: &Arc<tsox_compile::compiler::Program>,
        c: &mut Checker,
    ) -> Option<SignatureHelp> {
        if argument_info.invocation.contextual_invocation.is_some() {
            return None;
        }
        let expression = super::m5w::get_expression_from_invocation(argument_info)?;
        if !ast::is_property_access_expression(&expression) {
            return None;
        }
        let name = match &expression.data {
            tsox_frontend::ast::NodeData::PropertyAccessExpression(d) => d.name.text().to_string(),
            _ => return None,
        };
        if name.is_empty() {
            return None;
        }
        for sf in program.get_source_files() {
            if let Some(result) = self.find_signature_help_from_named_declarations(
                ctx,
                &sf,
                &name,
                argument_info,
                c,
            ) {
                return Some(result);
            }
        }
        None
    }

    pub fn find_signature_help_from_named_declarations(
        &self,
        ctx: &lsproto::Context,
        source_file: &Arc<SourceFile>,
        name: &str,
        argument_info: &M5wArgumentListInfo,
        c: &mut Checker,
    ) -> Option<SignatureHelp> {
        let mut result: Option<SignatureHelp> = None;
        m5v2_visit_named_declaration(
            self,
            ctx,
            &source_file.node,
            name,
            argument_info,
            source_file,
            c,
            &mut result,
        );
        result
    }

    pub fn compute_active_parameter(
        &self,
        sig: &M5v2SignatureInformation,
        argument_index: usize,
        supports_null: bool,
    ) -> Option<u32> {
        let param_count = sig.parameters.len();
        if param_count == 0 {
            return None;
        }

        let mut active_param = argument_index as u32;

        if sig.is_variadic {
            let first_rest = sig.parameters.iter().position(|p| p.is_variadic);
            if let Some(first_rest) = first_rest {
                if first_rest < param_count - 1 {
                    if supports_null {
                        return None;
                    }
                    return Some(param_count as u32);
                }
            }
            if active_param > (param_count - 1) as u32 {
                active_param = param_count as u32 - 1;
            }
        }

        Some(active_param)
    }
}

fn m5v2_visit_named_declaration(
    l: &LanguageService,
    ctx: &lsproto::Context,
    node: &Arc<Node>,
    name: &str,
    argument_info: &M5wArgumentListInfo,
    source_file: &Arc<SourceFile>,
    c: &mut Checker,
    result: &mut Option<SignatureHelp>,
) -> bool {
    if result.is_some() {
        return true;
    }
    if ast::mig::m3b::get_declaration_name(node) == name {
        if let Some(symbol) = c.get_symbol_of_node(node) {
            let t = c.get_type_of_symbol_at_location(&symbol, node);
            let call_signatures: Vec<Arc<Signature>> = c.get_call_signatures(&t);
            if !call_signatures.is_empty() {
                *result = l.create_signature_help_items(
                    ctx,
                    &call_signatures,
                    &call_signatures[0],
                    argument_info,
                    source_file,
                    c,
                    true,
                );
                if result.is_some() {
                    return true;
                }
            }
        }
    }
    let mut found = result.is_some();
    tsox_frontend::ast::node_data_generated::for_each_child(
        node,
        &mut |child: &Arc<Node>| {
            if m5v2_visit_named_declaration(l, ctx, child, name, argument_info, source_file, c, result)
            {
                found = true;
                return true;
            }
            false
        },
    );
    found
}
