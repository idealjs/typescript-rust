use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn references_for_string_literal_property_names2() {
    let content = r#"class Foo {
    /*1*/"/*2*/blah"() { return 0; }
}

var x: Foo;
x./*3*/blah;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
