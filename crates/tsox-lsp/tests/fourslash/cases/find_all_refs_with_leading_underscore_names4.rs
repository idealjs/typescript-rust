use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_with_leading_underscore_names4() {
    let content = r#"class Foo {
    /*1*/public /*2*/____bar() { return 0; }
}

var x: Foo;
x./*3*/____bar;"#;
    let mut s = Session::new_for_test("findAllRefsWithLeadingUnderscoreNames4", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
