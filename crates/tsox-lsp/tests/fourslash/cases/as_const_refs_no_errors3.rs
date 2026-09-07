use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn as_const_refs_no_errors3() {
    let content = r#"// @checkJs: true
// @Filename: file.js
class Tex {
    type = (/** @type {/**/const} */'Text');
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "")
    fourslash::verify_no_errors(&mut s);
}
