use tsox_lsp::fourslash::Session;


#[test]
fn find_all_references_from_link_tag_reference5() {
    let content = r#"enum E {
    /** {@link E./**/A} */
    A
}"#;
    let _s = Session::new_for_test("findAllReferencesFromLinkTagReference5", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
}
