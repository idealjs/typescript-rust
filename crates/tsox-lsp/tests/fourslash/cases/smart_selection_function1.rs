use tsox_lsp::fourslash::{self, Session};


#[test]
fn smart_selection_function1() {
    let content = r#"const f1 = () => {
   /**/
};"#;
    let mut s = Session::new_for_test("smartSelection_function1", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}
