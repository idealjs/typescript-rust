#![allow(dead_code, unused_imports, unused_variables)]

use super::m5h_5::create_diagnostic_for_node_in_source_file_or_compiler_diagnostic;
use crate::tsoptions::option_kind::{OptionDecl, OptionKind};
use tsox_core::diagnostics::ARGUMENT_FOR_0_OPTION_MUST_BE_COLON_1;
use tsox_frontend::ast::diagnostic::Diagnostic;
use tsox_frontend::ast::node_node::Node;
use tsox_frontend::ast::node_source_file::SourceFile;

impl OptionDecl {
    pub fn disallow_null_or_undefined(&self) -> bool { ::tsox_core::fntrace::enter("disallow_null_or_undefined"); 
        self.name == "extends"
    }
}

pub fn create_diagnostic_for_invalid_enum_type(
    opt: &OptionDecl,
    source_file: Option<&SourceFile>,
    node: Option<&Node>,
) -> Diagnostic { ::tsox_core::fntrace::enter("create_diagnostic_for_invalid_enum_type"); 
    let names_of_type: Vec<String> = opt
        .enum_map()
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();
    let string_names = format_enum_type_keys(opt, &names_of_type);
    let opt_name = format!("--{}", opt.name);
    create_diagnostic_for_node_in_source_file_or_compiler_diagnostic(
        source_file,
        node,
        ARGUMENT_FOR_0_OPTION_MUST_BE_COLON_1,
        vec![opt_name, string_names],
    )
}

pub fn format_enum_type_keys(opt: &OptionDecl, keys: &[String]) -> String { ::tsox_core::fntrace::enter("format_enum_type_keys"); 
    let keys: Vec<String> = match opt.deprecated_keys() {
        Some(deprecated) => keys
            .iter()
            .filter(|key| !deprecated.has(key))
            .cloned()
            .collect(),
        None => keys.to_vec(),
    };
    format!("'{}'", keys.join("', '"))
}

pub fn option_kind_name(kind: OptionKind) -> &'static str { ::tsox_core::fntrace::enter("option_kind_name"); 
    match kind {
        OptionKind::Boolean => "boolean",
        OptionKind::String => "string",
        OptionKind::Number => "number",
        OptionKind::List => "list",
        OptionKind::ListOrElement => "listOrElement",
        OptionKind::Enum => "enum",
    }
}

pub fn get_compiler_option_value_type_string(option: &OptionDecl) -> String { ::tsox_core::fntrace::enter("get_compiler_option_value_type_string"); 
    match option.kind {
        OptionKind::ListOrElement => format!(
            "{} or Array",
            option
                .elements()
                .map(get_compiler_option_value_type_string)
                .unwrap_or_default()
        ),
        OptionKind::List => "Array".to_string(),
        _ => option_kind_name(option.kind).to_string(),
    }
}
