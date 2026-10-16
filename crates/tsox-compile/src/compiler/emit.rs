#![allow(unused_imports, dead_code, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::SourceFile;

use crate::mig::m4v::Emitter;

pub fn emit_js_file_with(
    emitter: &mut Emitter,
    source_file: Option<Arc<SourceFile>>,
    js_file_path: &str,
    source_map_file_path: &str,
) { ::tsox_core::fntrace::enter("emit_js_file_with"); 
    emitter.emit_js_file(source_file, js_file_path, source_map_file_path)
}

pub fn emit_declaration_file_with(
    emitter: &mut Emitter,
    source_file: Option<Arc<SourceFile>>,
    declaration_file_path: &str,
    declaration_map_path: &str,
) { ::tsox_core::fntrace::enter("emit_declaration_file_with"); 
    emitter.emit_declaration_file(
        source_file,
        declaration_file_path,
        declaration_map_path,
    )
}
