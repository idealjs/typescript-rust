use tsox_lsp::fourslash::{self, Session};


#[test]
fn smart_selection_template_strings() {
    let content = r#"`a /*1*/b ${
  '/*2*/c'
} d`"#;
    let mut s = Session::new_for_test("smartSelection_templateStrings", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}
