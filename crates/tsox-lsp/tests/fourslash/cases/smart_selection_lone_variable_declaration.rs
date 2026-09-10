use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineSelectionRanges"]
#[test]
fn smart_selection_lone_variable_declaration() {
    let content = r#"const /**/x = 3;"#;
    let mut s = Session::new_for_test("smartSelection_loneVariableDeclaration", content);
    fourslash::unsupported("VerifyBaselineSelectionRanges"); // f.VerifyBaselineSelectionRanges(t)
}
