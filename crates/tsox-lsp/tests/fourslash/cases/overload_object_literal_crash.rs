use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyQuickInfoExists"]
#[test]
fn overload_object_literal_crash() {
    let content = r#"interface Foo {
    extend<T>(...objs: any[]): T;
    extend<T>(deep, target: T): T;
}
var $: Foo;
$.extend({ /**/foo: 0 }, "");
"#;
    let mut s = Session::new_for_test("overloadObjectLiteralCrash", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyQuickInfoExists"); // f.VerifyQuickInfoExists(t)
}
