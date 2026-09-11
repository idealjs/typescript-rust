use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsdoc_link_rename1() {
    let content = r#"interface A/**/ {}
/**
 * {@link A()} is ok
 */
declare const a: A"#;
    let mut s = Session::new_for_test("jsdocLink_rename1", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
