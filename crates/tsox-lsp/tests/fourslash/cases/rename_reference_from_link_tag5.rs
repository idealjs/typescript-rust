use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_reference_from_link_tag5() {
    let content = r#"enum E {
    /** {@link E./**/A} */
    A
}"#;
    let mut s = Session::new_for_test("renameReferenceFromLinkTag5", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
