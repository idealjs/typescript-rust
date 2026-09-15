use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_no_substitution_template_literal_no_crash1() {
    let content = r#"type Test = `T/*1*/`;"#;
    let _s = Session::new_for_test("findAllRefsNoSubstitutionTemplateLiteralNoCrash1", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
