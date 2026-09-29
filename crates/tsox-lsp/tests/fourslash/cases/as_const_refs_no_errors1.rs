use tsox_lsp::fourslash::{self, Session};


#[test]
fn as_const_refs_no_errors1() {
    let content = r#"class Tex {
    type = 'Text' as /**/const;
}"#;
    let mut s = Session::new_for_test("asConstRefsNoErrors1", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "")
    fourslash::verify_no_errors(&mut s, );
}
