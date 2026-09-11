use tsox_lsp::fourslash::{self, Session};


#[test]
fn smart_selection_function3() {
    let content = r#"const f3 = function () {
    /**/
}"#;
    let mut s = Session::new_for_test("smartSelection_function3", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}
