use super::m3n_5::r33k8_defs::DeclarationEmitHost;
use std::sync::Arc;
use tsox_core::core::compiler_options_kinds::ResolutionMode;
use tsox_core::core::text::TextRange;
use tsox_core::tspath::mig::m3i::get_relative_path_from_file;
use tsox_core::tspath::supported_ts_extensions_flat::ComparePathsOptions;
use tsox_frontend::ast::diagnostic::Diagnostic;
use tsox_frontend::ast::node_source_file::FileReference;
use tsox_frontend::ast::SourceFile;

pub struct SupplementalReferencesTransformer {
    host: DeclarationEmitHost,
    supplemental_files: Vec<Arc<SourceFile>>,
    declaration_file_path: String,
    force_declaration_paths: bool,
}

pub fn new_supplemental_references_transformer(
    host: DeclarationEmitHost,
    source_file: &SourceFile,
    declaration_file_path: String,
    force_declaration_paths: bool,
) -> SupplementalReferencesTransformer { ::tsox_core::fntrace::enter("new_supplemental_references_transformer"); 
    SupplementalReferencesTransformer {
        host,
        supplemental_files: source_file.supplemental_source_files(),
        declaration_file_path,
        force_declaration_paths,
    }
}

impl SupplementalReferencesTransformer {
    pub fn transform_source_file<'a>(
        &'a self,
        source_file: &'a mut SourceFile,
    ) -> &'a mut SourceFile { ::tsox_core::fntrace::enter("transform_source_file"); 
        for supplemental in &self.supplemental_files {
            if !self
                .host
                .source_file_may_be_emitted(supplemental, self.force_declaration_paths)
            {
                continue;
            }
            let declaration_path = self
                .host
                .get_output_paths_for(supplemental, self.force_declaration_paths)
                .declaration_file_path();
            if declaration_path.is_empty() {
                continue;
            }
            let relative = get_relative_path_from_file(
                &self.declaration_file_path,
                &declaration_path,
                &ComparePathsOptions {
                    current_directory: self.host.get_current_directory(),
                    use_case_sensitive_file_names: self.host.use_case_sensitive_file_names(),
                },
            );
            source_file.referenced_files.push(FileReference {
                range: TextRange { pos: -1, end: -1 },
                file_name: relative,
                resolution_mode: ResolutionMode::None,
                preserve: false,
            });
        }
        source_file
    }

    pub fn get_diagnostics(&self) -> Vec<Diagnostic> { ::tsox_core::fntrace::enter("get_diagnostics"); 
        Vec::new()
    }
}
