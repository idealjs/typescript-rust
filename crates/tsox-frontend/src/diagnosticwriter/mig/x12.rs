use std::io::Write;
use std::sync::Arc;

use crate::ast::diagnostic::Diagnostic;
use super::m5z::wrap_ast_diagnostic_owned;
use super::m5z_2::write_flattened_diagnostic_message;
use tsox_core::locale::Locale;

pub struct AstDiagnostic(pub Arc<Diagnostic>);

pub fn wrap_ast_diagnostic(d: Arc<Diagnostic>) -> AstDiagnostic { ::tsox_core::fntrace::enter("wrap_ast_diagnostic"); 
    AstDiagnostic(d)
}

pub fn wrap_ast_diagnostics(diags: Vec<Arc<Diagnostic>>) -> Vec<AstDiagnostic> { ::tsox_core::fntrace::enter("wrap_ast_diagnostics"); 
    diags.into_iter().map(wrap_ast_diagnostic).collect()
}

pub fn write_flattened_ast_diagnostic_message(
    writer: &mut dyn Write,
    diagnostic: &Diagnostic,
    newline: &str,
    locale: &Locale,
) { ::tsox_core::fntrace::enter("write_flattened_ast_diagnostic_message"); 
    write_flattened_diagnostic_message(
        writer,
        &wrap_ast_diagnostic_owned(diagnostic),
        newline,
        locale,
    );
}
