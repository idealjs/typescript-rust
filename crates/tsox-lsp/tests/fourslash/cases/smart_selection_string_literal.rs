use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineSelectionRanges"]
#[test]
fn smart_selection_string_literal() {
    let content = r#"const a = 'a';
const b = /**/'b';"#;
    let mut s = Session::new_for_test("smartSelection_stringLiteral", content);
    fourslash::unsupported("VerifyBaselineSelectionRanges"); // f.VerifyBaselineSelectionRanges(t)
}
