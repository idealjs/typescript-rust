use tsox_lsp::fourslash::Session;


#[test]
fn find_all_references_from_link_tag_reference1() {
    let content = r#"enum E {
    /** {@link /**/A} */
    A
}"#;
    let _s = Session::new_for_test("findAllReferencesFromLinkTagReference1", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
}
