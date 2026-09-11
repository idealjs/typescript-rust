use tsox_lsp::fourslash::{self, Session};


#[test]
fn export_assignment_missing_name() {
    // TODO: // Regression test for crash when export= has an incomplete/missing expression
    // TODO: // (e.g. "export = " with a trailing space). The SelectionRange must not fall
    // TODO: // outside the document symbol's Range.
    let content = r#"export = "#;
    let mut s = Session::new_for_test("exportAssignmentMissingName", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
