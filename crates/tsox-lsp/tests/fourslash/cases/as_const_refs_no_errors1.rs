use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn as_const_refs_no_errors1() {
    let content = r#"class Tex {
    type = 'Text' as /**/const;
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "")
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
}
