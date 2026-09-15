use tsox_lsp::fourslash::Session;


#[test]
fn references_for_string_literal_property_names3() {
    let content = r#"class Foo2 {
    /*1*/get "/*2*/42"() { return 0; }
    /*3*/set /*4*/42(n) { }
}

var y: Foo2;
y[/*5*/42];"#;
    let _s = Session::new_for_test("referencesForStringLiteralPropertyNames3", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5")
}
