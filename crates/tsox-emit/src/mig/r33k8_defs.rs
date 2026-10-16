use std::sync::Arc;
use tsox_checker::checker::mig::m2d::EmitResolver;
use tsox_frontend::ast::node_flags::ModifierFlags;
use tsox_frontend::ast::node_node::Node;
use tsox_frontend::ast::node_source_file::SourceFile;
use tsox_core::core::text::TextRange;
use tsox_core::core::compiler_options::ResolutionMode;
use tsox_frontend::ast::NodeData;

pub fn syntax_list_children(node: &Arc<Node>) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("syntax_list_children"); 
    match &node.data {
        NodeData::SyntaxList(data) => data.children.clone(),
        _ => Vec::new(),
    }
}

pub struct FileReference {
    pub file_name: String,
    pub text_range: TextRange,
    pub resolution_mode: Option<ResolutionMode>,
    pub preserve: bool,
}

pub type NodeId = u64;

pub struct OutputPathsValue {
    pub declaration_path: String,
    pub js_path: String,
    pub source_map_path: String,
    pub declaration_map_path: String,
}

impl OutputPathsValue {
    pub fn declaration_file_path(&self) -> String { ::tsox_core::fntrace::enter("declaration_file_path"); 
        self.declaration_path.clone()
    }

    pub fn js_file_path(&self) -> String { ::tsox_core::fntrace::enter("js_file_path"); 
        self.js_path.clone()
    }

    pub fn source_map_file_path(&self) -> String { ::tsox_core::fntrace::enter("source_map_file_path"); 
        self.source_map_path.clone()
    }

    pub fn declaration_map_path(&self) -> String { ::tsox_core::fntrace::enter("declaration_map_path"); 
        self.declaration_map_path.clone()
    }
}

#[derive(Clone)]
pub struct DeclarationEmitHost {
    get_current_directory_fn: Arc<dyn Fn() -> String + Send + Sync>,
    use_case_sensitive_file_names_fn: Arc<dyn Fn() -> bool + Send + Sync>,
    get_source_file_from_reference_fn:
        Arc<dyn Fn(&Arc<SourceFile>, &FileReference) -> Option<Arc<SourceFile>> + Send + Sync>,
    get_output_paths_for_fn: Arc<dyn Fn(&SourceFile, bool) -> OutputPathsValue + Send + Sync>,
    source_file_may_be_emitted_fn: Arc<dyn Fn(&SourceFile, bool) -> bool + Send + Sync>,
    get_effective_declaration_flags_fn:
        Arc<dyn Fn(&Arc<Node>, ModifierFlags) -> ModifierFlags + Send + Sync>,
    get_emit_resolver_fn: Arc<dyn Fn() -> EmitResolver + Send + Sync>,
}

pub struct DeclarationEmitHostFns {
    pub get_current_directory: Arc<dyn Fn() -> String + Send + Sync>,
    pub use_case_sensitive_file_names: Arc<dyn Fn() -> bool + Send + Sync>,
    pub get_source_file_from_reference:
        Arc<dyn Fn(&Arc<SourceFile>, &FileReference) -> Option<Arc<SourceFile>> + Send + Sync>,
    pub get_output_paths_for: Arc<dyn Fn(&SourceFile, bool) -> OutputPathsValue + Send + Sync>,
    pub source_file_may_be_emitted: Arc<dyn Fn(&SourceFile, bool) -> bool + Send + Sync>,
    pub get_effective_declaration_flags:
        Arc<dyn Fn(&Arc<Node>, ModifierFlags) -> ModifierFlags + Send + Sync>,
    pub get_emit_resolver: Arc<dyn Fn() -> EmitResolver + Send + Sync>,
}

impl DeclarationEmitHost {
    pub fn new(fns: DeclarationEmitHostFns) -> Self { ::tsox_core::fntrace::enter("new"); 
        Self {
            get_current_directory_fn: fns.get_current_directory,
            use_case_sensitive_file_names_fn: fns.use_case_sensitive_file_names,
            get_source_file_from_reference_fn: fns.get_source_file_from_reference,
            get_output_paths_for_fn: fns.get_output_paths_for,
            source_file_may_be_emitted_fn: fns.source_file_may_be_emitted,
            get_effective_declaration_flags_fn: fns.get_effective_declaration_flags,
            get_emit_resolver_fn: fns.get_emit_resolver,
        }
    }

    pub fn get_current_directory(&self) -> String { ::tsox_core::fntrace::enter("get_current_directory"); 
        (self.get_current_directory_fn)()
    }

    pub fn use_case_sensitive_file_names(&self) -> bool { ::tsox_core::fntrace::enter("use_case_sensitive_file_names"); 
        (self.use_case_sensitive_file_names_fn)()
    }

    pub fn get_source_file_from_reference(
        &self,
        origin: &Arc<SourceFile>,
        reference: &FileReference,
    ) -> Option<Arc<SourceFile>> { ::tsox_core::fntrace::enter("get_source_file_from_reference"); 
        (self.get_source_file_from_reference_fn)(origin, reference)
    }

    pub fn get_output_paths_for(&self, file: &SourceFile, force_dts_paths: bool) -> OutputPathsValue { ::tsox_core::fntrace::enter("get_output_paths_for"); 
        (self.get_output_paths_for_fn)(file, force_dts_paths)
    }

    pub fn source_file_may_be_emitted(&self, file: &SourceFile, force_dts_emit: bool) -> bool { ::tsox_core::fntrace::enter("source_file_may_be_emitted"); 
        (self.source_file_may_be_emitted_fn)(file, force_dts_emit)
    }

    pub fn get_effective_declaration_flags(
        &self,
        node: &Arc<Node>,
        flags: ModifierFlags,
    ) -> ModifierFlags { ::tsox_core::fntrace::enter("get_effective_declaration_flags"); 
        (self.get_effective_declaration_flags_fn)(node, flags)
    }

    pub fn get_emit_resolver(&self) -> EmitResolver { ::tsox_core::fntrace::enter("get_emit_resolver"); 
        (self.get_emit_resolver_fn)()
    }
}
