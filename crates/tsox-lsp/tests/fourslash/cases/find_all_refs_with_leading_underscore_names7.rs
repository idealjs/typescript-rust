use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_with_leading_underscore_names7() {
    let content = r#"/*1*/function /*2*/__foo() {
    /*3*/__foo();
}"#;
    let _s = Session::new_for_test("findAllRefsWithLeadingUnderscoreNames7", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
