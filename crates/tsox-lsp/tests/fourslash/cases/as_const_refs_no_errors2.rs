use tsox_lsp::fourslash::{self, Session};


#[test]
fn as_const_refs_no_errors2() {
    let content = r#"class Tex {
    type = </**/const>'Text';
}"#;
    let mut s = Session::new_for_test("asConstRefsNoErrors2", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "")
    fourslash::verify_no_errors(&mut s, );
}
