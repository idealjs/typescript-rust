use tsox_lsp::fourslash::Session;


#[test]
fn find_all_references_of_constructor_bad_overload() {
    let content = r#"class C {
    /*1*/constructor(n: number);
    /*2*/constructor(){}
}"#;
    let _s = Session::new_for_test("findAllReferencesOfConstructor_badOverload", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
}
