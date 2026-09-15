use tsox_lsp::fourslash::Session;


#[test]
fn rename_reference_from_link_tag1() {
    let content = r#"enum E {
    /** {@link /**/A} */
    A
}"#;
    let _s = Session::new_for_test("renameReferenceFromLinkTag1", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
