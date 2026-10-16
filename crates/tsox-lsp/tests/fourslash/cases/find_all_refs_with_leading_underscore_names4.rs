use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_with_leading_underscore_names4() {
    let content = r#"class Foo {
    /*1*/public /*2*/____bar() { return 0; }
}

var x: Foo;
x./*3*/____bar;"#;
    let _s = Session::new_for_test("findAllRefsWithLeadingUnderscoreNames4", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
