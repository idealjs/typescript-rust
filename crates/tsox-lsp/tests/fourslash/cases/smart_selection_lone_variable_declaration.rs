use tsox_lsp::fourslash::{self, Session};


#[test]
fn smart_selection_lone_variable_declaration() {
    let content = r#"const /**/x = 3;"#;
    let mut s = Session::new_for_test("smartSelection_loneVariableDeclaration", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}
