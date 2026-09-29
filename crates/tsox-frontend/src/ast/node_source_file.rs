use crate::ast::diagnostic::Diagnostic;
use crate::ast::node_line_map::LineMap;
use crate::ast::node_node::Node;
use std::sync::Arc;

fn bind_diagnostics_store(
) -> &'static std::sync::RwLock<std::collections::HashMap<u64, Vec<Arc<Diagnostic>>>> {
    static STORE: std::sync::OnceLock<
        std::sync::RwLock<std::collections::HashMap<u64, Vec<Arc<Diagnostic>>>>,
    > = std::sync::OnceLock::new();
    STORE.get_or_init(|| std::sync::RwLock::new(std::collections::HashMap::new()))
}

#[derive(Debug)]
pub struct SourceFile {
    pub node: Arc<Node>,
    pub file_name: String,
    pub text: String,
    pub line_map: LineMap,
    pub language_variant: LanguageVariant,
    pub script_kind: ScriptKind,

    pub comment_directives: Vec<crate::scanner::CommentDirective>,

    pub jsdoc_cache: std::sync::RwLock<std::collections::HashMap<u64, Vec<Arc<Node>>>>,

    pub has_lazy_jsdoc: bool,

    pub is_declaration_file: bool,

    pub imports: Vec<Arc<Node>>,

    pub module_augmentations: Vec<Arc<Node>>,

    pub ambient_module_names: Vec<String>,

    pub parse_error_spans: Vec<tsox_core::core::text::TextRange>,

    pub external_module_indicator: Option<Arc<Node>>,

    pub common_js_module_indicator: Option<Arc<Node>>,

    pub uses_uri_style_node_core_modules: tsox_core::core::tristate::Tristate,

    pub has_parse_diagnostics: bool,

    pub referenced_files: Vec<FileReference>,

    pub type_reference_directives: Vec<FileReference>,

    pub lib_reference_directives: Vec<FileReference>,

    pub supplemental_source_files: Vec<std::sync::Arc<SourceFile>>,
}

#[derive(Debug, Clone)]
pub struct FileReference {
    pub range: tsox_core::core::text::TextRange,
    pub file_name: String,
    pub resolution_mode: tsox_core::core::compiler_options_kinds::ResolutionMode,
    pub preserve: bool,
}

impl SourceFile {
    pub fn id(&self) -> u64 {
        self.node.id()
    }

    pub fn bind_diagnostics(&self) -> Vec<Arc<Diagnostic>> {
        bind_diagnostics_store()
            .read()
            .unwrap()
            .get(&self.id())
            .cloned()
            .unwrap_or_default()
    }

    pub fn set_bind_diagnostics(&self, diags: Vec<Arc<Diagnostic>>) {
        bind_diagnostics_store()
            .write()
            .unwrap()
            .insert(self.id(), diags);
    }

    pub fn supplemental_source_files(&self) -> Vec<std::sync::Arc<SourceFile>> {
        self.supplemental_source_files.clone()
    }

    pub fn set_jsdoc_cache(&self, cache: std::collections::HashMap<u64, Vec<Arc<Node>>>) {
        *self.jsdoc_cache.write().unwrap() = cache;
    }

    pub fn set_has_lazy_jsdoc(&mut self, lazy: bool) {
        self.has_lazy_jsdoc = lazy;
    }

    pub fn has_lazy_jsdoc(&self) -> bool {
        self.has_lazy_jsdoc
    }

    pub fn resolve_jsdoc(&self, node: &Node) -> Vec<Arc<Node>> {
        let node_id = node.id();

        {
            let cache = self.jsdoc_cache.read().unwrap();
            if let Some(jsdocs) = cache.get(&node_id) {
                return jsdocs.clone();
            }
        }

        let mut cache = self.jsdoc_cache.write().unwrap();
        if let Some(jsdocs) = cache.get(&node_id) {
            return jsdocs.clone();
        }
        let jsdocs = crate::parser::parse_jsdoc_for_node(self, node);
        cache.insert(node_id, jsdocs.clone());
        jsdocs
    }

    pub fn eager_jsdoc(&self, node: &Node) -> Vec<Arc<Node>> {
        let cache = self.jsdoc_cache.read().unwrap();
        cache.get(&node.id()).cloned().unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LanguageVariant {
    #[default]
    Standard,
    Jsx,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScriptKind {
    #[default]
    Unknown,
    Js,
    Jsx,
    Ts,
    Tsx,
    Json,
    External,
    Deferred,
}
