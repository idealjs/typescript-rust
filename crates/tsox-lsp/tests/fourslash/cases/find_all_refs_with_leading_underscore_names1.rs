use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_with_leading_underscore_names1() {
    let content = r#"class Foo {
    /*1*/public /*2*/_bar() { return 0; }
}

var x: Foo;
x./*3*/_bar;"#;
    let mut s = Session::new_for_test("findAllRefsWithLeadingUnderscoreNames1", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
