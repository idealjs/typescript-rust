use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_no_substitution_template_literal_no_crash1() {
    let content = r#"type Test = ` + "`" + `T/*1*/` + "`" + `;"#;
    let mut s = Session::new_for_test("findAllRefsNoSubstitutionTemplateLiteralNoCrash1", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1")
}
