use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_no_default_lib() {
    let content = r#"// @checkJs: true
// @allowJs: true
// @Filename: /foo.js
// @ts-check
const [|/**/foo|] = 1;"#;
    let mut s = Session::new_for_test("renameNoDefaultLib", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyRenameSucceeded(t, nil /*preferences*/)
}
