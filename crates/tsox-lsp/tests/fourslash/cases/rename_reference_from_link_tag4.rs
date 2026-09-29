use tsox_lsp::fourslash::Session;


#[test]
fn rename_reference_from_link_tag4() {
    let content = r#"enum E {
    /** {@link /**/B} */
    A,
    B
}"#;
    let _s = Session::new_for_test("renameReferenceFromLinkTag4", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
