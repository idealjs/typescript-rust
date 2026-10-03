#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::text::{TextPos, TextRange};
use tsox_frontend::ast::SourceFile;

use crate::ls::source_map::DocumentPosition;

// script 结构体孪生:m5u_conv::OriginalTextScript 已覆盖 FileName/OriginalFileName/Text/OriginalText/SpanMap
// (span_map 恒 None),本批不重复定义,source_map 族函数直接以 crate::mig::m5u_conv::OriginalTextScript 为 script。

impl crate::ls::language_service::LanguageService {
    pub fn source_file_range_to_lsp_location(
        &self,
        file: &Arc<SourceFile>,
        file_range: TextRange,
    ) -> (crate::lsp::lsproto_lsp::Location, crate::ls::mig::m5s::SpanFidelity) { ::tsox_core::fntrace::enter("source_file_range_to_lsp_location"); 
        if !source_file_content_mapper(file).is_empty() {
            let script_view = crate::mig::m5u_conv::SourceFileScriptView { file: Arc::clone(file) };
            let (range, fidelity) = m5w4_m5u_converters().to_lsp_range_for_feature(
                &script_view,
                file_range,
                0,
            );
            return (
                crate::lsp::lsproto_lsp::Location {
                    uri: crate::lsp::lsproto_lsp::DocumentUri(
                        crate::ls::lsconv_converters::file_name_to_document_uri(
                            &script_view.file.file_name,
                        ),
                    ),
                    range,
                },
                m5w4_span_fidelity(fidelity),
            );
        }
        (
            self.get_mapped_location(&file.file_name, file_range),
            crate::ls::mig::m5s::SpanFidelity::None_,
        )
    }

    pub fn source_file_range_to_lsp_location_for_feature(
        &self,
        file: &Arc<SourceFile>,
        file_range: TextRange,
        feature: crate::ls::mig::m5s::SpanFeature,
    ) -> (crate::lsp::lsproto_lsp::Location, crate::ls::mig::m5s::SpanFidelity) { ::tsox_core::fntrace::enter("source_file_range_to_lsp_location_for_feature"); 
        if !source_file_content_mapper(file).is_empty() {
            let script_view = crate::mig::m5u_conv::SourceFileScriptView { file: Arc::clone(file) };
            let (range, fidelity) = m5w4_m5u_converters().to_lsp_range_for_feature(
                &script_view,
                file_range,
                m5w4_span_feature(feature),
            );
            return (
                crate::lsp::lsproto_lsp::Location {
                    uri: crate::lsp::lsproto_lsp::DocumentUri(
                        crate::ls::lsconv_converters::file_name_to_document_uri(
                            &script_view.file.file_name,
                        ),
                    ),
                    range,
                },
                m5w4_span_fidelity(fidelity),
            );
        }
        (
            self.get_mapped_location(&file.file_name, file_range),
            crate::ls::mig::m5s::SpanFidelity::None_,
        )
    }

    pub fn m5w_try_get_generated_position(
        &self,
        file_name: &str,
        position: TextPos,
    ) -> Option<DocumentPosition> { ::tsox_core::fntrace::enter("m5w_try_get_generated_position"); 
        let new_pos = self.m5w_try_get_generated_position_worker(file_name, position)?;
        if self.read_file(&new_pos.file_name).is_none() {
            return None;
        }
        Some(new_pos)
    }

    pub fn m5w_try_get_generated_position_worker(
        &self,
        file_name: &str,
        position: TextPos,
    ) -> Option<DocumentPosition> { ::tsox_core::fntrace::enter("m5w_try_get_generated_position_worker"); 
        if tsox_core::tspath::is_declaration_file_name(file_name) {
            return None;
        }

        let program = self.get_program();
        if program.get_source_file(file_name).is_none() {
            return None;
        }

        let path = self.to_path(file_name);
        if program.is_source_from_project_reference(&path.0) {
            return None;
        }

        let declaration_file_name = get_output_declaration_file_name_worker(
            file_name,
            program.options(),
        );
        let mapper = crate::mig::m6b::get_document_position_mapper(
            &M6bHostAdapter(self.host.as_ref()),
            &declaration_file_name,
        )?;
        let mapped = mapper.get_generated_position(&crate::mig::m6b::DocumentPosition {
            file_name: file_name.to_string(),
            pos: position,
        })?;
        let document_pos = DocumentPosition {
            file_name: mapped.file_name,
            pos: mapped.pos,
        };
        if let Some(new_pos) =
            self.m5w_try_get_generated_position_worker(&document_pos.file_name, document_pos.pos)
        {
            return Some(new_pos);
        }
        Some(document_pos)
    }

    pub fn try_get_source_position_worker(
        &self,
        file_name: &str,
        position: TextPos,
    ) -> Option<DocumentPosition> { ::tsox_core::fntrace::enter("try_get_source_position_worker"); 
        if !tsox_core::tspath::is_declaration_file_name(file_name) {
            return None;
        }

        let mapper =
            crate::mig::m6b::get_document_position_mapper(&M6bHostAdapter(self.host.as_ref()), file_name)?;
        let mapped = mapper.get_source_position(&crate::mig::m6b::DocumentPosition {
            file_name: file_name.to_string(),
            pos: position,
        })?;
        let document_pos = DocumentPosition {
            file_name: mapped.file_name,
            pos: mapped.pos,
        };
        if let Some(new_pos) =
            self.try_get_source_position_worker(&document_pos.file_name, document_pos.pos)
        {
            return Some(new_pos);
        }
        Some(document_pos)
    }

    pub fn m5w_get_script(
        &self,
        file_name: &str,
    ) -> Option<crate::mig::m5u_conv::OriginalTextScript> { ::tsox_core::fntrace::enter("m5w_get_script"); 
        let text = self.read_file(file_name)?;
        Some(crate::mig::m5u_conv::original_text_script(
            file_name.to_string(),
            text,
        ))
    }
}

pub fn source_file_content_mapper(file: &Arc<SourceFile>) -> String { ::tsox_core::fntrace::enter("source_file_content_mapper"); 
    tsox_compile::mig::m3l_cm_2::content_mapper_source_file_info(&file.file_name)
        .map(|info| info.content_mapper)
        .unwrap_or_default()
}

fn m5w4_m5u_converters() -> crate::mig::m5u_conv::M5uConverters { ::tsox_core::fntrace::enter("m5w4_m5u_converters"); 
    crate::mig::m5u_conv::new_converters(
        crate::ls::lsconv_converters::PositionEncodingKind::Utf16,
        Box::new(crate::ls::lsconv_linemap::compute_lsp_line_starts),
    )
}

fn m5w4_span_feature(feature: crate::ls::mig::m5s::SpanFeature) -> crate::mig::m5u_conv::SpanMapFeature { ::tsox_core::fntrace::enter("m5w4_span_feature"); 
    use crate::ls::mig::m5s::SpanFeature as F;
    match feature {
        F::Definition => crate::mig::m6b_2::FEATURE_DEFINITION as u32,
        F::TypeDefinition => crate::mig::m6b_2::FEATURE_TYPE_DEFINITION as u32,
        F::DocumentHighlights => crate::mig::m6b_2::FEATURE_DOCUMENT_HIGHLIGHTS as u32,
        F::References => crate::mig::m6b_2::FEATURE_REFERENCES as u32,
        F::Implementation => crate::mig::m6b_2::FEATURE_IMPLEMENTATION as u32,
    }
}

fn m5w4_span_fidelity(fidelity: crate::mig::m5u_conv::SpanMapFidelity) -> crate::ls::mig::m5s::SpanFidelity { ::tsox_core::fntrace::enter("m5w4_span_fidelity"); 
    use crate::ls::mig::m5s::SpanFidelity as F;
    use crate::mig::m5u_conv::SPANMAP_FIDELITY_EXACT;
    use crate::mig::m5u_conv::SPANMAP_FIDELITY_SINGLE_SEGMENT;
    if fidelity == SPANMAP_FIDELITY_EXACT {
        F::Exact
    } else if fidelity == SPANMAP_FIDELITY_SINGLE_SEGMENT {
        F::SingleSegment
    } else {
        F::None_
    }
}

struct M6bHostAdapter<'a>(&'a dyn crate::ls::host::Host);

impl crate::mig::m6b::Host for M6bHostAdapter<'_> {
    fn use_case_sensitive_file_names(&self) -> bool { ::tsox_core::fntrace::enter("use_case_sensitive_file_names"); 
        self.0.use_case_sensitive_file_names()
    }
    fn get_ecma_line_info(&self, file_name: &str) -> Option<crate::mig::m6b::EcmaLineInfo> { ::tsox_core::fntrace::enter("get_ecma_line_info"); 
        let text = self.0.read_file(file_name)?;
        let line_map = crate::ls::lsconv_linemap::compute_lsp_line_starts(&text);
        Some(crate::mig::m6b::create_ecma_line_info(
            text,
            line_map.line_starts.into_iter().map(|p| p as i32).collect(),
        ))
    }
    fn read_file(&self, file_name: &str) -> Option<String> { ::tsox_core::fntrace::enter("read_file"); 
        self.0.read_file(file_name)
    }
}

pub fn get_output_declaration_file_name_worker(
    input_file_name: &str,
    options: &CompilerOptions,
) -> String { ::tsox_core::fntrace::enter("get_output_declaration_file_name_worker"); 
    let dir = if options.declaration_dir.is_empty() {
        &options.out_dir
    } else {
        &options.declaration_dir
    };
    if dir.is_empty() {
        return change_to_declaration_extension(input_file_name);
    }
    let current_directory = tsox_core::tspath::get_directory_path(&options.config_file_path);
    let relative = tsox_core::tspath::mig::m3i::get_relative_path_from_directory(
        &current_directory,
        input_file_name,
        &tsox_core::tspath::ComparePathsOptions {
            use_case_sensitive_file_names: true,
            current_directory: current_directory.clone(),
        },
    );
    let output = tsox_core::tspath::resolve_path(dir, &[relative.as_str()]);
    change_to_declaration_extension(&output)
}

fn change_to_declaration_extension(file_name: &str) -> String { ::tsox_core::fntrace::enter("change_to_declaration_extension"); 
    use tsox_core::tspath as tsp;
    if tsp::file_extension_is_one_of(
        file_name,
        &[tsp::EXTENSION_MTS, tsp::EXTENSION_MJS],
    ) {
        tsp::change_extension(file_name, tsp::EXTENSION_DMTS)
    } else if tsp::file_extension_is_one_of(
        file_name,
        &[tsp::EXTENSION_CTS, tsp::EXTENSION_CJS],
    ) {
        tsp::change_extension(file_name, tsp::EXTENSION_DCTS)
    } else if tsp::file_extension_is(file_name, tsp::EXTENSION_JSX) {
        tsp::change_extension(file_name, ".d.jsx")
    } else {
        tsp::change_extension(file_name, tsp::EXTENSION_DTS)
    }
}
