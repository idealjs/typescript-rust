use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_references_from_link_tag_reference5() {
    let content = r#"enum E {
    /** {@link E./**/A} */
    A
}"#;
    let mut s = Session::new_for_test("findAllReferencesFromLinkTagReference5", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "")
}
