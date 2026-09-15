use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_with_leading_underscore_names6() {
    let content = r#"class Foo {
    public _bar;
    /*1*/public /*2*/__bar;
    public ___bar;
    public ____bar;
}

var x: Foo;
x._bar;
x./*3*/__bar;
x.___bar;
x.____bar;"#;
    let _s = Session::new_for_test("findAllRefsWithLeadingUnderscoreNames6", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
