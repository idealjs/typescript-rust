use tsox_lsp::fourslash::Session;


#[test]
fn find_all_references_from_link_tag_reference4() {
    let content = r#"enum E {
    /** {@link /**/B} */
    A,
    B
}"#;
    let _s = Session::new_for_test("findAllReferencesFromLinkTagReference4", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
}
