#![allow(unused_imports)]

use super::m4v::{FileIncludeKind, ProcessingDiagnostic, ProcessingDiagnosticKind};
use crate::compiler::Program;
use std::sync::Arc;
use tsox_core::diagnostics::messages_generated as messages;
use tsox_core::tspath;

impl ProcessingDiagnostic {
    pub fn to_diagnostic(&self, program: &Program) -> Option<Arc<tsox_frontend::ast::diagnostic::Diagnostic>> { ::tsox_core::fntrace::enter("to_diagnostic"); 
        match self.kind {
            ProcessingDiagnosticKind::UnknownReference => {
                self.create_unknown_reference_diagnostic(program).map(Arc::new)
            }
            ProcessingDiagnosticKind::ExplainingFileInclude => {
                self.create_diagnostic_explaining_file(program).map(Arc::new)
            }
        }
    }

    fn create_unknown_reference_diagnostic(
        &self,
        program: &Program,
    ) -> Option<tsox_frontend::ast::diagnostic::Diagnostic> { ::tsox_core::fntrace::enter("create_unknown_reference_diagnostic"); 
        let r = self.as_file_include_reason();
        let loc = r.get_referenced_location(program);
        let file_name = loc.ref_.as_ref().unwrap().file_name.clone();
        match r.kind {
            FileIncludeKind::TypeReferenceDirective => loc.diagnostic_at(
                messages::CANNOT_FIND_TYPE_DEFINITION_FILE_FOR_0,
                vec![file_name],
            ),
            FileIncludeKind::LibReferenceDirective => {
                let lib_name = tspath::to_file_name_lower_case(&file_name);
                let unqualified_lib_name = lib_name
                    .strip_prefix("lib.")
                    .unwrap_or(&lib_name)
                    .strip_suffix(".d.ts")
                    .unwrap_or(&lib_name)
                    .to_string();
                let suggestion = tsox_core::core::mig::m3j_2::get_spelling_suggestion_for_strings(
                    &unqualified_lib_name,
                    tsox_tsoptions::mig::m5h_5::LIB_MAP
                        .iter()
                        .map(|(name, _)| name.to_string()),
                );
                if let Some(suggestion) = suggestion {
                    loc.diagnostic_at(
                        messages::CANNOT_FIND_LIB_DEFINITION_FOR_0_DID_YOU_MEAN_1,
                        vec![lib_name.clone(), suggestion],
                    )
                } else {
                    loc.diagnostic_at(
                        messages::CANNOT_FIND_LIB_DEFINITION_FOR_0,
                        vec![lib_name.clone()],
                    )
                }
            }
            _ => panic!("unknown include kind"),
        }
    }
}
