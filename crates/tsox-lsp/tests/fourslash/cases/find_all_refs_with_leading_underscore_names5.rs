use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_with_leading_underscore_names5() {
    let content = r#"class Foo {
    public _bar;
    public __bar;
    /*1*/public /*2*/___bar;
    public ____bar;
}

var x: Foo;
x._bar;
x.__bar;
x./*3*/___bar;
x.____bar;"#;
    let mut s = Session::new_for_test("findAllRefsWithLeadingUnderscoreNames5", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
