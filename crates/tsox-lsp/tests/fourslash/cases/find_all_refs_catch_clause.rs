use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_catch_clause() {
    let content = r#"try { }
catch (/*1*/err) {
    /*2*/err;
}"#;
    let _s = Session::new_for_test("findAllRefsCatchClause", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
}
