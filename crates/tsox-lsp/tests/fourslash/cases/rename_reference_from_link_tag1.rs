use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_reference_from_link_tag1() {
    let content = r#"enum E {
    /** {@link /**/A} */
    A
}"#;
    let mut s = Session::new_for_test("renameReferenceFromLinkTag1", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
