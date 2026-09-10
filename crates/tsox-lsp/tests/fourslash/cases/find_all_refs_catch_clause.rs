use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_catch_clause() {
    let content = r#"try { }
catch (/*1*/err) {
    /*2*/err;
}"#;
    let mut s = Session::new_for_test("findAllRefsCatchClause", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2")
}
