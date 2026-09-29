use tsox_lsp::fourslash::Session;


#[test]
fn references_for_string_literal_property_names4() {
    let content = r#"var x = { "/*1*/someProperty": 0 }
x[/*2*/"someProperty"] = 3;
x.someProperty = 5;"#;
    let _s = Session::new_for_test("referencesForStringLiteralPropertyNames4", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
}
