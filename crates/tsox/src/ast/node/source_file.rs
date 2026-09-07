use super::line_map::LineMap;
use super::node::Node;
use std::sync::Arc;

#[derive(Debug)]
pub struct SourceFile {
    pub node: Arc<Node>,
    pub file_name: String,
    pub text: String,
    pub line_map: LineMap,
    pub language_variant: LanguageVariant,
    pub script_kind: ScriptKind,

    pub comment_directives: Vec<crate::scanner::CommentDirective>,

    pub(crate) jsdoc_cache: std::sync::RwLock<std::collections::HashMap<u64, Vec<Arc<Node>>>>,

    pub(crate) has_lazy_jsdoc: bool,

    pub is_declaration_file: bool,

    pub imports: Vec<Arc<Node>>,

    pub module_augmentations: Vec<Arc<Node>>,

    pub ambient_module_names: Vec<String>,

    pub parse_error_spans: Vec<crate::core::text::TextRange>,

    pub external_module_indicator: Option<Arc<Node>>,

    pub common_js_module_indicator: Option<Arc<Node>>,

    pub uses_uri_style_node_core_modules: crate::core::tristate::Tristate,

    pub has_parse_diagnostics: bool,
}

impl SourceFile {
    pub fn id(&self) -> u64 {
        self.node.id()
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
