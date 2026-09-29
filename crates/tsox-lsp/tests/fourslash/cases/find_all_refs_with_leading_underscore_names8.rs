use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_with_leading_underscore_names8() {
    let content = r#"(/*1*/function /*2*/__foo() {
    /*3*/__foo();
})"#;
    let _s = Session::new_for_test("findAllRefsWithLeadingUnderscoreNames8", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
