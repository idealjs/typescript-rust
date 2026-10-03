#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::{Arc, OnceLock};

use tsox_core::core::text::TextRange;
use tsox_frontend::ast::{Node, SourceFile};

pub enum CallHierarchyDeclarationResult {
    Node(Arc<Node>),
    Nodes(Vec<Arc<Node>>),
}

pub struct IncomingEntry {
    pub ls: crate::ls::language_service::LanguageService,
    pub node: Arc<Node>,
    source_file: OnceLock<Option<Arc<SourceFile>>>,
    document_uri: OnceLock<String>,
    position: OnceLock<crate::lsp::lsproto::Position>,
}

impl IncomingEntry {
    pub fn new(ls: crate::ls::language_service::LanguageService, node: Arc<Node>) -> Self { ::tsox_core::fntrace::enter("new"); 
        IncomingEntry {
            ls,
            node,
            source_file: OnceLock::new(),
            document_uri: OnceLock::new(),
            position: OnceLock::new(),
        }
    }

    pub fn get_source_file(&self) -> Option<Arc<SourceFile>> { ::tsox_core::fntrace::enter("get_source_file"); 
        self.source_file
            .get_or_init(|| {
                tsox_frontend::ast::get_source_file_of_node(&self.node)
                    .and_then(|n| crate::ls::mig::m5u::node_as_source_file(&n))
            })
            .clone()
    }

    pub fn text_document_uri(&self) -> String { ::tsox_core::fntrace::enter("text_document_uri"); 
        self.document_uri
            .get_or_init(|| {
                crate::ls::lsconv_converters::file_name_to_document_uri(
                    &self
                        .get_source_file()
                        .map(|f| f.file_name.clone())
                        .unwrap_or_default(),
                )
            })
            .clone()
    }

    pub fn text_document_position(&self) -> crate::lsp::lsproto::Position { ::tsox_core::fntrace::enter("text_document_position"); 
        self.position
            .get_or_init(|| {
                let Some(f) = self.get_source_file() else {
                    return crate::lsp::lsproto::Position::default();
                };
                let start = tsox_frontend::scanner::mig::x5a::get_token_pos_of_node(&self.node, &f, false);
                let script = crate::mig::m5u_conv::PlainScriptView {
                    file_name: f.file_name.clone(),
                    text: f.text.clone(),
                };
                self.ls
                    .converters
                    .position_to_line_and_character(&script, start)
            })
            .clone()
    }
}

pub struct CallSite {
    pub declaration: Arc<Node>,
    pub text_range: TextRange,
    pub source_file: Arc<SourceFile>,
}

pub struct CallSiteCollector {
    pub program: Arc<tsox_compile::compiler::Program>,
    pub call_sites: Vec<CallSite>,
}
