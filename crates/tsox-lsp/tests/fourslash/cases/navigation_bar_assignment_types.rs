use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_bar_assignment_types() {
    let content = r#"'use strict'
const a = {
    ...b,
    c,
    d: 0
};"#;
    let mut s = Session::new_for_test("navigationBarAssignmentTypes", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
