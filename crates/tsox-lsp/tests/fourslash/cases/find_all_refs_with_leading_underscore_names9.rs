use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_with_leading_underscore_names9() {
    let content = r#"(/*1*/function /*2*/___foo() {
    /*3*/___foo();
})"#;
    let _s = Session::new_for_test("findAllRefsWithLeadingUnderscoreNames9", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
