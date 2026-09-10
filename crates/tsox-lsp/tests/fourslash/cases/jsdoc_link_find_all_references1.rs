use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn jsdoc_link_find_all_references1() {
    let content = r#"interface A/**/ {}
/**
 * {@link A()} is ok
 */
declare const a: A"#;
    let mut s = Session::new_for_test("jsdocLink_findAllReferences1", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "")
}
