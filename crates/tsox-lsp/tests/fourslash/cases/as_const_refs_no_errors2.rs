use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn as_const_refs_no_errors2() {
    let content = r#"class Tex {
    type = </**/const>'Text';
}"#;
    let mut s = Session::new_for_test("asConstRefsNoErrors2", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "")
    fourslash::verify_no_errors(&mut s, );
}
