use tsox_lsp::fourslash::Session;


#[test]
fn rename_reference_from_link_tag5() {
    let content = r#"enum E {
    /** {@link E./**/A} */
    A
}"#;
    let _s = Session::new_for_test("renameReferenceFromLinkTag5", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
