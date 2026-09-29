#![allow(unused_imports)]

#[allow(unused_imports, ambiguous_glob_reexports)]
use crate::checker::*;
#[allow(unused_imports)]
use tsox_core::diagnostics::messages_generated::*;
use std::sync::Arc;

use crate::checker::mig::m3a_2::DiagnosticDetails;
use tsox_core::tspath::directory_separator::combine_paths;
use tsox_core::tspath::get_normalized_absolute_path::file_extension_is;
use tsox_core::tspath::supported_ts_extensions_flat::try_get_extension_from_path;
use tsox_tsoptions::module::{get_types_package_name, mangle_scoped_package_name};

pub fn create_module_not_found_chain_details(
    program: &dyn crate::checker::checker_heritage_retry_limit::Program,
    file: Option<&Arc<tsox_frontend::ast::SourceFile>>,
    module_reference: &str,
    _mode: tsox_core::core::compiler_options_kinds::ResolutionMode,
    package_name: &str,
) -> DiagnosticDetails {
    let resolved = file.and_then(|f| program.get_resolved_module(&f.file_name, module_reference));
    if let Some(resolved) = resolved {
        let package_name = if resolved.contains("/node_modules/@types/") {
            format!("@types/{}", mangle_scoped_package_name(package_name))
        } else {
            package_name.to_string()
        };
        return DiagnosticDetails {
            message: THERE_ARE_TYPES_AT_0_BUT_THIS_RESULT_COULD_NOT_BE_RESOLVED_WHEN_RESPECTING_PACKAGE_JSON_EXPORTS_THE_1_LIBRARY_MAY_NEED_TO_UPDATE_ITS_PACKAGE_JSON_OR_TYPINGS,
            args: vec![resolved, package_name],
        };
    }
    DiagnosticDetails {
        message: TRY_NPM_I_SAVE_DEV_TYPES_SLASH_1_IF_IT_EXISTS_OR_ADD_A_NEW_DECLARATION_D_TS_FILE_CONTAINING_DECLARE_MODULE_0,
        args: vec![module_reference.to_string(), mangle_scoped_package_name(package_name)],
    }
}

pub fn create_mode_mismatch_details_worker(
    _program: &dyn crate::checker::checker_heritage_retry_limit::Program,
    file: &Arc<tsox_frontend::ast::SourceFile>,
) -> DiagnosticDetails {
    let ext = try_get_extension_from_path(&file.file_name);
    let target_ext: &str = if ext == ".ts" {
        ".mts"
    } else if ext == ".js" {
        ".mjs"
    } else {
        ""
    };
    let package_json_type = "";
    let package_json_directory = "";
    if !package_json_directory.is_empty() && package_json_type.is_empty() {
        if !target_ext.is_empty() {
            return DiagnosticDetails {
                message: TO_CONVERT_THIS_FILE_TO_AN_ECMASCRIPT_MODULE_CHANGE_ITS_FILE_EXTENSION_TO_0_OR_ADD_THE_FIELD_TYPE_COLON_MODULE_TO_1,
                args: vec![
                    target_ext.to_string(),
                    combine_paths(package_json_directory, &["package.json"]),
                ],
            };
        }
        return DiagnosticDetails {
            message: TO_CONVERT_THIS_FILE_TO_AN_ECMASCRIPT_MODULE_ADD_THE_FIELD_TYPE_COLON_MODULE_TO_0,
            args: vec![combine_paths(package_json_directory, &["package.json"])],
        };
    }
    if !target_ext.is_empty() {
        return DiagnosticDetails {
            message: TO_CONVERT_THIS_FILE_TO_AN_ECMASCRIPT_MODULE_CHANGE_ITS_FILE_EXTENSION_TO_0_OR_CREATE_A_LOCAL_PACKAGE_JSON_FILE_WITH_TYPE_COLON_MODULE,
            args: vec![target_ext.to_string()],
        };
    }
    DiagnosticDetails {
        message: TO_CONVERT_THIS_FILE_TO_AN_ECMASCRIPT_MODULE_CREATE_A_LOCAL_PACKAGE_JSON_FILE_WITH_TYPE_COLON_MODULE,
        args: vec![],
    }
}
