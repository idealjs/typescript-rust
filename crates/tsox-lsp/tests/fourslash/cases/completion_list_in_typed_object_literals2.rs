use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_typed_object_literals2() {
    let content = r#"interface Foo {
    x: { a: number };
}
var aaa: Foo;
aaa = { /*9*/"#;
    let mut s = Session::new_for_test("completionListInTypedObjectLiterals2", content);
    fourslash::verify_completions_exact_at(&mut s, Some("9"), &["x"]);
    // TODO: }
}
