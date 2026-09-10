use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_reference_from_link_tag4() {
    let content = r#"enum E {
    /** {@link /**/B} */
    A,
    B
}"#;
    let mut s = Session::new_for_test("renameReferenceFromLinkTag4", content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
