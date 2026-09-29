#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashMap;
use std::sync::Arc;

use crate::ls::autoimport::AutoImportFixKind;
use crate::ls::autoimport_fix::Fix;
use crate::ls::autoimport_import_adder::{ImportAdder, ImportAdderTrait};
use crate::ls::autoimport_view::{QueryKind, View};
use crate::ls::code_actions::{CodeFixContext, CombinedCodeActions};
use crate::ls::types::CodeAction;
use crate::ls::lsutil_user_preferences::UserPreferences;
use tsox_checker::checker::Checker;
use tsox_compile::compiler::Program;
use tsox_core::core::text::TextRange;
use tsox_core::diagnostics;
use tsox_frontend::ast::{self, Node, SourceFile, Symbol};
use tsox_frontend::astnav;
use tsox_frontend::scanner;

fn m5q_m5u_converters() -> crate::mig::m5u_conv::M5uConverters {
    crate::mig::m5u_conv::new_converters(
        crate::ls::lsconv_converters::PositionEncodingKind::Utf16,
        Box::new(crate::ls::lsconv_linemap::compute_lsp_line_starts),
    )
}

pub const IMPORT_FIX_ID: &str = "fixMissingImport";

pub fn import_fix_error_codes() -> Vec<i32> {
    vec![
        diagnostics::CANNOT_FIND_NAME_0.code(),
        diagnostics::CANNOT_FIND_NAME_0_DID_YOU_MEAN_1.code(),
        diagnostics::CANNOT_FIND_NAME_0_DID_YOU_MEAN_THE_INSTANCE_MEMBER_THIS_0.code(),
        diagnostics::CANNOT_FIND_NAME_0_DID_YOU_MEAN_THE_STATIC_MEMBER_1_0.code(),
        diagnostics::CANNOT_FIND_NAMESPACE_0.code(),
        diagnostics::X_0_REFERS_TO_A_UMD_GLOBAL_BUT_THE_CURRENT_FILE_IS_A_MODULE_CONSIDER_ADDING_AN_IMPORT_INSTEAD.code(),
        diagnostics::X_0_ONLY_REFERS_TO_A_TYPE_BUT_IS_BEING_USED_AS_A_VALUE_HERE.code(),
        diagnostics::NO_VALUE_EXISTS_IN_SCOPE_FOR_THE_SHORTHAND_PROPERTY_0_EITHER_DECLARE_ONE_OR_PROVIDE_AN_INITIALIZER.code(),
        diagnostics::X_0_CANNOT_BE_USED_AS_A_VALUE_BECAUSE_IT_WAS_IMPORTED_USING_IMPORT_TYPE.code(),
        diagnostics::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_JQUERY_TRY_NPM_I_SAVE_DEV_TYPES_SLASHJQUERY.code(),
        diagnostics::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_CHANGE_YOUR_TARGET_LIBRARY_TRY_CHANGING_THE_LIB_COMPILER_OPTION_TO_1_OR_LATER.code(),
        diagnostics::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_CHANGE_YOUR_TARGET_LIBRARY_TRY_CHANGING_THE_LIB_COMPILER_OPTION_TO_INCLUDE_DOM.code(),
        diagnostics::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_A_TEST_RUNNER_TRY_NPM_I_SAVE_DEV_TYPES_SLASHJEST_OR_NPM_I_SAVE_DEV_TYPES_SLASHMOCHA_AND_THEN_ADD_JEST_OR_MOCHA_TO_THE_TYPES_FIELD_IN_YOUR_TSCONFIG.code(),
        diagnostics::CANNOT_FIND_NAME_0_DID_YOU_MEAN_TO_WRITE_THIS_IN_AN_ASYNC_FUNCTION.code(),
        diagnostics::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_JQUERY_TRY_NPM_I_SAVE_DEV_TYPES_SLASHJQUERY_AND_THEN_ADD_JQUERY_TO_THE_TYPES_FIELD_IN_YOUR_TSCONFIG.code(),
        diagnostics::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_A_TEST_RUNNER_TRY_NPM_I_SAVE_DEV_TYPES_SLASHJEST_OR_NPM_I_SAVE_DEV_TYPES_SLASHMOCHA.code(),
        diagnostics::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_NODE_TRY_NPM_I_SAVE_DEV_TYPES_SLASHNODE.code(),
        diagnostics::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_NODE_TRY_NPM_I_SAVE_DEV_TYPES_SLASHNODE_AND_THEN_ADD_NODE_TO_THE_TYPES_FIELD_IN_YOUR_TSCONFIG.code(),
        diagnostics::CANNOT_FIND_NAMESPACE_0_DID_YOU_MEAN_1.code(),
        diagnostics::CANNOT_EXTEND_AN_INTERFACE_0_DID_YOU_MEAN_IMPLEMENTS.code(),
        diagnostics::THIS_JSX_TAG_REQUIRES_0_TO_BE_IN_SCOPE_BUT_IT_COULD_NOT_BE_FOUND.code(),
    ]
}

#[derive(Debug, Clone)]
pub struct FixInfo {
    pub fix: Fix,
    pub symbol_name: String,
    pub error_identifier_text: String,
    pub is_jsx_namespace_fix: bool,
}

pub fn is_fixable_diagnostic(diag: &ast::Diagnostic, error_codes: &[i32]) -> bool {
    error_codes.contains(&diag.code())
}

pub fn add_import_from_diagnostic(
    import_adder: &mut ImportAdder,
    diag: &ast::Diagnostic,
    fix_context: &CodeFixContext,
) -> Result<(), String> {
    let diag_fix_context = CodeFixContext {
        source_file: fix_context.source_file,
        span: TextRange::new(diag.pos().max(0) as usize, diag.end().max(0) as usize),
        error_code: diag.code(),
        program: fix_context.program,
        ls: fix_context.ls,
        diagnostic: None,
        params: None,
    };

    let infos = get_fix_infos(&diag_fix_context, diag.code(), diag.pos())?;
    if !infos.is_empty() {
        import_adder.add_import_fix(&infos[0].fix);
    }
    Ok(())
}

pub fn get_fix_infos(
    fix_context: &CodeFixContext,
    error_code: i32,
    pos: i32,
) -> Result<Vec<FixInfo>, String> {
    if tsox_core::tspath::is_dynamic_file_name(&fix_context.source_file.file_name) {
        return Ok(Vec::new());
    }

    let symbol_token = match astnav::get_token_at_position(&fix_context.source_file.node, pos.max(0) as usize) {
        Some(token) => token,
        None => return Ok(Vec::new()),
    };

    let mut view: Option<View> = None;
    let mut info: Vec<FixInfo> = Vec::new();

    if error_code
        == diagnostics::X_0_REFERS_TO_A_UMD_GLOBAL_BUT_THE_CURRENT_FILE_IS_A_MODULE_CONSIDER_ADDING_AN_IMPORT_INSTEAD.code()
    {
        view = Some(fix_context.ls.get_current_auto_import_view(fix_context.source_file));
        info = get_fixes_info_for_umd_import(fix_context, &symbol_token, view.as_ref().unwrap());
    } else if !ast::is_identifier(&symbol_token) {
        return Ok(Vec::new());
    } else if error_code
        == diagnostics::X_0_CANNOT_BE_USED_AS_A_VALUE_BECAUSE_IT_WAS_IMPORTED_USING_IMPORT_TYPE.code()
    {
        let ch = fix_context.program.get_type_checker();
        let compiler_options = fix_context.program.options();
        let symbol_names =
            get_symbol_names_to_import(fix_context.source_file, &ch, &symbol_token, compiler_options);

        let mut all_type_only_fixes: Vec<FixInfo> = Vec::new();
        for sn in &symbol_names {
            if !sn.is_type_only {
                continue;
            }
            let fix = get_type_only_promotion_fix(
                fix_context.source_file,
                &symbol_token,
                &sn.name,
                fix_context.program,
            );
            if let Some(fix) = fix {
                all_type_only_fixes.push(FixInfo {
                    fix,
                    symbol_name: sn.name.clone(),
                    error_identifier_text: symbol_token.text().to_string(),
                    is_jsx_namespace_fix: false,
                });
            }
        }

        let mut diagnostic_message = String::new();
        if let Some(diag) = fix_context.diagnostic {
            diagnostic_message = diag.message.clone();
        }
        if all_type_only_fixes.len() > 1 && !diagnostic_message.is_empty() {
            for fi in &all_type_only_fixes {
                if diagnostic_message.contains(&format!("'{}'", fi.symbol_name)) {
                    info.push(fi.clone());
                }
            }
        }
        if info.is_empty() {
            info = all_type_only_fixes;
        }
        return Ok(info);
    } else {
        match fix_context.ls.get_prepared_auto_import_view(fix_context.source_file) {
            Ok(v) => {
                info = get_fixes_info_for_non_umd_import(fix_context, &symbol_token, &v);
                view = Some(v);
            }
            Err(_) => {}
        }
    }

    if view.is_none() {
        view = Some(fix_context.ls.get_current_auto_import_view(fix_context.source_file));
    }
    Ok(sort_fix_info(info, fix_context, view.as_ref()))
}

pub fn get_fixes_info_for_umd_import(
    fix_context: &CodeFixContext,
    token: &Arc<Node>,
    view: &View,
) -> Vec<FixInfo> {
    let ch = fix_context.program.get_type_checker();

    let umd_symbol = get_umd_symbol(token, &ch);
    let umd_symbol = match umd_symbol {
        Some(s) => s,
        None => return Vec::new(),
    };

    let export = crate::ls::autoimport_export::symbol_to_export(&umd_symbol, &ch);
    let export = match export {
        Some(e) => e,
        None => return Vec::new(),
    };
    let is_valid_type_only_use_site = tsox_frontend::ast::mig::m3g_3::is_valid_type_only_alias_use_site(token);

    let mut result = Vec::new();
    for fix in view.get_fixes(&export, false, is_valid_type_only_use_site, None) {
        let error_identifier_text = if ast::is_identifier(token) {
            token.text().to_string()
        } else {
            String::new()
        };
        result.push(FixInfo {
            fix,
            symbol_name: umd_symbol.name.clone(),
            error_identifier_text,
            is_jsx_namespace_fix: false,
        });
    }
    result
}

pub fn get_umd_symbol(token: &Arc<Node>, ch: &Checker) -> Option<Arc<Symbol>> {
    let mut umd_symbol: Option<Arc<Symbol>> = None;
    if ast::is_identifier(token) {
        umd_symbol = ch.get_resolved_symbol(token);
    }
    if is_umd_export_symbol(umd_symbol.as_deref()) {
        return umd_symbol;
    }

    let parent = token.parent()?;
    let parent_is_jsx_opening_like = tsox_frontend::ast::mig::m3g::is_jsx_opening_like_element(&parent);
    if (parent_is_jsx_opening_like && Arc::ptr_eq(tsox_frontend::ast::mig::m3c::tag_name(&parent), token))
        || ast::is_jsx_opening_fragment(&parent)
    {
        let location = if parent_is_jsx_opening_like {
            token.clone()
        } else {
            parent.clone()
        };
        let jsx_namespace = ch.get_jsx_namespace_str(&parent);
        let parent_symbol = ch.resolve_name(
            &jsx_namespace,
            &location,
            ast::SymbolFlags::VALUE,
            false,
        );
        if is_umd_export_symbol(parent_symbol.as_deref()) {
            return parent_symbol;
        }
    }
    None
}

pub fn is_umd_export_symbol(symbol: Option<&Symbol>) -> bool {
    match symbol {
        Some(symbol) => {
            !symbol.declarations.is_empty()
                && ast::is_namespace_export_declaration(&symbol.declarations[0])
        }
        None => false,
    }
}

pub fn get_fixes_info_for_non_umd_import(
    fix_context: &CodeFixContext,
    symbol_token: &Arc<Node>,
    view: &View,
) -> Vec<FixInfo> {
    let ch = fix_context.program.get_type_checker();
    let compiler_options = fix_context.program.options();

    let is_valid_type_only_use_site =
        tsox_frontend::ast::mig::m3g_3::is_valid_type_only_alias_use_site(symbol_token);
    let symbol_names =
        get_symbol_names_to_import(fix_context.source_file, &ch, symbol_token, compiler_options);
    let mut all_info: Vec<FixInfo> = Vec::new();

    let script_view = crate::mig::m5u_conv::SourceFileScriptView {
        file: Arc::clone(fix_context.source_file),
    };
    let (usage_position, fidelity) = m5q_m5u_converters().to_lsp_position(
        &script_view,
        tsox_frontend::scanner::mig::x5a::get_token_pos_of_node(
            symbol_token,
            fix_context.source_file.as_ref(),
            false,
        ) as usize,
    );
    if !crate::ls::mig::m5u_3::fidelity_is_exact(&fidelity) {
        return Vec::new();
    }

    for sn in &symbol_names {
        if sn.is_type_only {
            continue;
        }

        let symbol_name = sn.name.as_str();
        if symbol_name == "default" {
            continue;
        }

        let is_jsx_tag_name = symbol_name == symbol_token.text() && ast::is_jsx_tag_name(symbol_token);
        let mut query_kind = QueryKind::ExactMatch;
        if is_jsx_tag_name {
            query_kind = QueryKind::CaseInsensitiveMatch;
        }

        let exports = view.search(symbol_name, query_kind);
        for export in exports {
            if is_jsx_tag_name && !(export.name() == symbol_name || export.is_renameable()) {
                continue;
            }

            let fixes = view.get_fixes(
                &export,
                is_jsx_tag_name,
                is_valid_type_only_use_site,
                Some(&usage_position),
            );
            for fix in fixes {
                all_info.push(FixInfo {
                    fix,
                    symbol_name: symbol_name.to_string(),
                    error_identifier_text: String::new(),
                    is_jsx_namespace_fix: symbol_name != symbol_token.text(),
                });
            }
        }
    }

    all_info
}

pub fn get_type_only_promotion_fix(
    source_file: &Arc<SourceFile>,
    symbol_token: &Arc<Node>,
    symbol_name: &str,
    program: &Program,
) -> Option<Fix> {
    let ch = program.get_type_checker();

    let symbol = ch.resolve_name(symbol_name, symbol_token, ast::SymbolFlags::VALUE, true)?;
    let type_only_alias_declaration = ch.get_type_only_alias_declaration(&symbol)?;
    match ast::get_source_file_of_node(&type_only_alias_declaration) {
        Some(n) if Arc::ptr_eq(&n, &source_file.node) => {}
        _ => return None,
    }

    Some(Fix {
        auto_import_fix: crate::ls::autoimport::AutoImportFix {
            kind: AutoImportFixKind::PromoteTypeOnly,
            ..Default::default()
        },
        type_only_alias_declaration: Some(type_only_alias_declaration),
        ..Default::default()
    })
}

#[derive(Debug, Clone)]
pub struct SymbolNameInfo {
    pub name: String,
    pub is_type_only: bool,
}

pub fn get_symbol_names_to_import(
    source_file: &Arc<SourceFile>,
    ch: &Checker,
    symbol_token: &Arc<Node>,
    compiler_options: &tsox_core::core::compiler_options::CompilerOptions,
) -> Vec<SymbolNameInfo> {
    if let Some(parent) = symbol_token.parent() {
    if (tsox_frontend::ast::mig::m3g::is_jsx_opening_like_element(&parent) || ast::is_jsx_closing_element(&parent))
        && Arc::ptr_eq(tsox_frontend::ast::mig::m3c::tag_name(&parent), symbol_token)
        && jsx_mode_needs_explicit_import(compiler_options.jsx)
    {
        let jsx_namespace = ch.get_jsx_namespace_str(&source_file.node);
        if needs_jsx_namespace_fix(&jsx_namespace, symbol_token, ch) {
            let mut result: Vec<SymbolNameInfo> = Vec::new();
            if !tsox_frontend::scanner::mig::m3i::is_intrinsic_jsx_name(&symbol_token.text()) {
                let comp_symbol =
                    ch.resolve_name(&symbol_token.text(), symbol_token, ast::SymbolFlags::VALUE, false);
                match comp_symbol {
                    None => result.push(SymbolNameInfo {
                        name: symbol_token.text().to_string(),
                        is_type_only: false,
                    }),
                    Some(comp_symbol) => {
                        if ch.get_type_only_alias_declaration(&comp_symbol).is_some() {
                            result.push(SymbolNameInfo {
                                name: symbol_token.text().to_string(),
                                is_type_only: true,
                            });
                        }
                    }
                }
            }
            let mut ns_is_type_only = false;
            if let Some(ns_symbol) =
                ch.resolve_name(&jsx_namespace, symbol_token, ast::SymbolFlags::VALUE, true)
            {
                ns_is_type_only = ch.get_type_only_alias_declaration(&ns_symbol).is_some();
            }
            result.push(SymbolNameInfo {
                name: jsx_namespace,
                is_type_only: ns_is_type_only,
            });
            return result;
        }
    }
    }
    let mut token_is_type_only = false;
    if let Some(sym) =
        ch.resolve_name(&symbol_token.text(), symbol_token, ast::SymbolFlags::VALUE, true)
    {
        token_is_type_only = ch.get_type_only_alias_declaration(&sym).is_some();
    }
    vec![SymbolNameInfo {
        name: symbol_token.text().to_string(),
        is_type_only: token_is_type_only,
    }]
}

pub fn needs_jsx_namespace_fix(jsx_namespace: &str, symbol_token: &Arc<Node>, ch: &Checker) -> bool {
    if tsox_frontend::scanner::mig::m3i::is_intrinsic_jsx_name(&symbol_token.text()) {
        return true;
    }
    let namespace_symbol =
        ch.resolve_name(jsx_namespace, symbol_token, ast::SymbolFlags::VALUE, true);
    let namespace_symbol = match namespace_symbol {
        Some(s) => s,
        None => return true,
    };
    if namespace_symbol
        .declarations
        .iter()
        .any(|d| tsox_frontend::ast::mig::m3g_2::is_type_only_import_or_export_declaration(d))
    {
        !namespace_symbol.flags.intersects(ast::SymbolFlags::VALUE)
    } else {
        false
    }
}

pub fn jsx_mode_needs_explicit_import(
    jsx: tsox_core::core::compiler_options_kinds::JsxEmit,
) -> bool {
    jsx == tsox_core::core::compiler_options_kinds::JsxEmit::React
        || jsx == tsox_core::core::compiler_options_kinds::JsxEmit::ReactNative
}

pub fn sort_fix_info(
    mut fixes: Vec<FixInfo>,
    fix_context: &CodeFixContext,
    view: Option<&View>,
) -> Vec<FixInfo> {
    if fixes.is_empty() {
        return fixes;
    }

    if let Some(view) = view {
        fixes.sort_by(|a, b| {
            let cmp = tsox_core::core::mig::m3j_2::compare_booleans(
                a.is_jsx_namespace_fix,
                b.is_jsx_namespace_fix,
            );
            if cmp != 0 {
                return cmp.cmp(&0);
            }
            view.compare_fixes_for_sorting(&a.fix, &b.fix)
        });
    }
    fixes
}
