use tsox_lsp::fourslash::Session;


#[test]
fn references_for_string_literal_property_names() {
    let content = r#"class Foo {
    public "/*1*/ss": any;
}

var x: Foo;
x.ss;
x["ss"];
x = { "ss": 0 };
x = { ss: 0 };"#;
    let _s = Session::new_for_test("referencesForStringLiteralPropertyNames", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
