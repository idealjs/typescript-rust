use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_of_constructor_with_modifier() {
    let content = r#"class X {
    public /*0*/constructor() {}
}
var x = new X();"#;
    let _s = Session::new_for_test("findAllRefsOfConstructor_withModifier", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "0")
}
