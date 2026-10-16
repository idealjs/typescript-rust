use crate::binder::binder::Binder;
use std::sync::Arc;
use tsox_core::diagnostics::messages_generated as msg;
use tsox_core::diagnostics::Message;
use tsox_frontend::ast::get_containing_class;
use tsox_frontend::ast::{Node, SourceFile, Symbol, SymbolFlags, SyntaxKind};

pub trait SymbolFlagsExt {
    const All: SymbolFlags;
    const Value: SymbolFlags;
    const Variable: SymbolFlags;
    const Enum: SymbolFlags;
    const Accessor: SymbolFlags;
}

impl SymbolFlagsExt for SymbolFlags {
    const All: SymbolFlags = SymbolFlags::all().difference(SymbolFlags::GlobalLookup);
    const Value: SymbolFlags = SymbolFlags::VALUE;
    const Variable: SymbolFlags = SymbolFlags::VARIABLE;
    const Enum: SymbolFlags = SymbolFlags::ENUM;
    const Accessor: SymbolFlags = SymbolFlags::ACCESSOR;
}

pub trait SourceFileNodeExt {
    fn as_node(&self) -> Arc<Node>;
}

impl SourceFileNodeExt for SourceFile {
    fn as_node(&self) -> Arc<Node> { ::tsox_core::fntrace::enter("as_node"); 
        self.node.clone()
    }
}

impl SourceFileNodeExt for Arc<SourceFile> {
    fn as_node(&self) -> Arc<Node> { ::tsox_core::fntrace::enter("as_node"); 
        self.node.clone()
    }
}

pub fn is_eval_or_arguments_identifier(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_eval_or_arguments_identifier"); 
    node.kind == SyntaxKind::Identifier
        && (node.text() == "eval" || node.text() == "arguments")
}

pub fn set_export_symbol(local: &Arc<Symbol>, export_symbol: Arc<Symbol>) { ::tsox_core::fntrace::enter("set_export_symbol"); 
    let ptr = Arc::as_ptr(local) as *mut Symbol;
    unsafe {
        (*ptr).export_symbol = Some(export_symbol);
    }
}

pub fn set_local_symbol_of_exportable(_node: &Arc<Node>, _local: Arc<Symbol>) { ::tsox_core::fntrace::enter("set_local_symbol_of_exportable"); }

pub fn set_symbol_flags_or(symbol: &Arc<Symbol>, flags: SymbolFlags) { ::tsox_core::fntrace::enter("set_symbol_flags_or"); 
    let ptr = Arc::as_ptr(symbol) as *mut Symbol;
    unsafe {
        (*ptr).flags = (*ptr).flags.union(flags);
    }
}

pub fn export_assignment_is_export_equals(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("export_assignment_is_export_equals"); 
    match &node.data {
        tsox_frontend::ast::NodeData::ExportAssignment(data) => data.is_export_equals,
        _ => false,
    }
}

pub fn export_specifier_name(node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("export_specifier_name"); 
    match &node.data {
        tsox_frontend::ast::NodeData::ExportSpecifier(data) => data
            .property_name
            .clone()
            .unwrap_or_else(|| data.name.clone()),
        _ => node.clone(),
    }
}

impl Binder {
    pub(crate) fn get_strict_mode_eval_or_arguments_message(&self, node: &Arc<Node>) -> &'static Message { ::tsox_core::fntrace::enter("get_strict_mode_eval_or_arguments_message"); 
        if get_containing_class(node).is_some() {
            return &msg::CODE_CONTAINED_IN_A_CLASS_IS_EVALUATED_IN_JAVASCRIPT_S_STRICT_MODE_WHICH_DOES_NOT_ALLOW_THIS_USE_OF_0_FOR_MORE_INFORMATION_SEE_HTTPS_COLON_SLASH_SLASHDEVELOPER_MOZILLA_ORG_SLASHEN_US_SLASHDOCS_SLASHWEB_SLASHJAVASCRIPT_SLASHREFERENCE_SLASHSTRICT_MODE;
        }
        if let Some(file) = self.current_source_file.as_ref() {
            if file.external_module_indicator.is_some() {
                return &msg::INVALID_USE_OF_0_MODULES_ARE_AUTOMATICALLY_IN_STRICT_MODE;
            }
        }
        &msg::INVALID_USE_OF_0_IN_STRICT_MODE
    }
}
