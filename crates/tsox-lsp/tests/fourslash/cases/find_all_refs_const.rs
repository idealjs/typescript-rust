use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_const() {
    let content = r#"// @Filename: a.ts
/**/const const
"#;
    let _s = Session::new_for_test("findAllRefsConst", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
}
