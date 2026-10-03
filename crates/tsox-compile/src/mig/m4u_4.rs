use std::sync::Arc;

use tsox_core::core::text::TextPos;
use tsox_frontend::ast::SourceFile;

pub use super::m4v::Emitter;

pub struct DeclarationMapSource {
    pub file_name: String,
    pub text: String,
    pub line_map: Vec<TextPos>,
}

pub fn new_declaration_map_source(source_file: &Arc<SourceFile>) -> DeclarationMapSource { ::tsox_core::fntrace::enter("new_declaration_map_source"); 
    use tsox_frontend::ast::mig::m3b_2;
    let text = m3b_2::original_text(source_file);
    DeclarationMapSource {
        file_name: m3b_2::original_file_name(source_file).to_string(),
        text: text.to_string(),
        line_map: tsox_core::core::mig::m3j::compute_ecma_line_starts(text),
    }
}

impl DeclarationMapSource {
    pub fn file_name(&self) -> &str { ::tsox_core::fntrace::enter("file_name"); 
        &self.file_name
    }

    pub fn text(&self) -> &str { ::tsox_core::fntrace::enter("text"); 
        &self.text
    }

    pub fn ecma_line_map(&self) -> &[TextPos] { ::tsox_core::fntrace::enter("ecma_line_map"); 
        &self.line_map
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EmitOnly {
    EmitAll,
    EmitOnlyJs,
    EmitOnlyDts,
    EmitOnlyBuilderSignature,
}

pub struct WriteFileData {
    pub source_map_url_pos: usize,
    pub build_info: Option<serde_json::Value>,
    pub diagnostics: Vec<Arc<tsox_frontend::ast::Diagnostic>>,
    pub skipped_dts_write: bool,
    pub source_file: Option<Arc<SourceFile>>,
}

impl Emitter {
    pub fn emit(&mut self) { ::tsox_core::fntrace::enter("emit"); 
        let Some(source_file) = self.source_file.clone() else {
            return;
        };
        if let Some(paths) = self.paths.as_ref() {
            let js_file_path = paths.js_file_path();
            let source_map_file_path = paths.source_map_file_path();
            let declaration_file_path = paths.declaration_file_path();
            let declaration_map_path = paths.declaration_map_path();
            crate::compiler::emit_js_file_with(
                self,
                Some(source_file.clone()),
                &js_file_path,
                &source_map_file_path,
            );
            crate::compiler::emit_declaration_file_with(
                self,
                Some(source_file.clone()),
                &declaration_file_path,
                &declaration_map_path,
            );
        }
        self.emit_result.diagnostics = self.take_emitter_diagnostics_formatted();
    }

    fn take_emitter_diagnostics_formatted(&mut self) -> Vec<String> { ::tsox_core::fntrace::enter("take_emitter_diagnostics_formatted"); 
        self.emitter_diagnostics
            .get_all()
            .iter()
            .map(|d| tsox_frontend::diagnosticwriter::format_diagnostic_compact(d, None))
            .collect()
    }
}
