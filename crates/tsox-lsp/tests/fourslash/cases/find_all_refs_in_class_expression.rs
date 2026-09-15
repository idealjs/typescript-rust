use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_in_class_expression() {
    let content = r#"interface I { /*0*/boom(): void; }
new class C implements I {
   /*1*/boom(){}
}"#;
    let _s = Session::new_for_test("findAllRefsInClassExpression", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "0", "1")
}
