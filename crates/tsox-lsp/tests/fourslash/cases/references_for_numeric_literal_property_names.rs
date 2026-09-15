use tsox_lsp::fourslash::Session;


#[test]
fn references_for_numeric_literal_property_names() {
    let content = r#"class Foo {
    public /*1*/12: any;
}

var x: Foo;
x[12];
x = { "12": 0 };
x = { 12: 0 };"#;
    let _s = Session::new_for_test("referencesForNumericLiteralPropertyNames", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
