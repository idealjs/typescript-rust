use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
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
    let mut s = Session::new_for_test("referencesForStringLiteralPropertyNames", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1")
}
