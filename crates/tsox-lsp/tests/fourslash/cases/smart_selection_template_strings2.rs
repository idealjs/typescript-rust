use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineSelectionRanges"]
#[test]
fn smart_selection_template_strings2() {
    let content = r#"` + "`" + `a ${b} /**/c` + "`" + `"#;
    let mut s = Session::new_for_test("smartSelection_templateStrings2", content);
    fourslash::unsupported("VerifyBaselineSelectionRanges"); // f.VerifyBaselineSelectionRanges(t)
}
