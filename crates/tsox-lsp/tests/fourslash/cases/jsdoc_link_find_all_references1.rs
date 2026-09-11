use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsdoc_link_find_all_references1() {
    let content = r#"interface A/**/ {}
/**
 * {@link A()} is ok
 */
declare const a: A"#;
    let mut s = Session::new_for_test("jsdocLink_findAllReferences1", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
}
