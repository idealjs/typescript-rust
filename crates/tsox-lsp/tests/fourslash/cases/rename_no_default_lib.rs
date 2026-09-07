use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyRenameSucceeded"]
#[test]
fn rename_no_default_lib() {
    let content = r#"// @checkJs: true
// @allowJs: true
// @Filename: /foo.js
// @ts-check
const [|/**/foo|] = 1;"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyRenameSucceeded"); // f.VerifyRenameSucceeded(t, nil /*preferences*/)
}
