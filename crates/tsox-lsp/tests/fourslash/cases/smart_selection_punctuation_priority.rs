use tsox_lsp::fourslash::{self, Session};


#[test]
fn smart_selection_punctuation_priority() {
    let content = r#"console/**/.log();"#;
    let mut s = Session::new_for_test("smartSelection_punctuationPriority", content);
    // TODO: f.VerifyBaselineSelectionRanges(t)
}
