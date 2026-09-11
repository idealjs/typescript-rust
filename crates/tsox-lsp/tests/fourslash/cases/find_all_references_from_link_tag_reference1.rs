use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_references_from_link_tag_reference1() {
    let content = r#"enum E {
    /** {@link /**/A} */
    A
}"#;
    let mut s = Session::new_for_test("findAllReferencesFromLinkTagReference1", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
}
