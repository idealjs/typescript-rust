use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyRenameSucceeded"]
#[test]
fn rename_for_aliasing_export01() {
    let content = r#"// @Filename: foo.ts
let x = 1;

export { /**/[|x|] as y };"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyRenameSucceeded"); // f.VerifyRenameSucceeded(t, nil /*preferences*/)
}
