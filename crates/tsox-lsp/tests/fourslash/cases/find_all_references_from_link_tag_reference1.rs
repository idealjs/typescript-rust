use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_references_from_link_tag_reference1() {
    let content = r#"enum E {
    /** {@link /**/A} */
    A
}"#;
    let mut s = Session::new_for_test("findAllReferencesFromLinkTagReference1", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "")
}
