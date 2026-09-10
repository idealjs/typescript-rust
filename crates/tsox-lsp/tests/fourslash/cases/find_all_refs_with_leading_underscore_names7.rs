use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_with_leading_underscore_names7() {
    let content = r#"/*1*/function /*2*/__foo() {
    /*3*/__foo();
}"#;
    let mut s = Session::new_for_test("findAllRefsWithLeadingUnderscoreNames7", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
