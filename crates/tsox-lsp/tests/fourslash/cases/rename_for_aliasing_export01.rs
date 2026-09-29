use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_for_aliasing_export01() {
    let content = r#"// @Filename: foo.ts
let x = 1;

export { /**/[|x|] as y };"#;
    let mut s = Session::new_for_test("renameForAliasingExport01", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyRenameSucceeded(t, nil /*preferences*/)
}
