use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: }"]
#[test]
fn completion_list_in_typed_object_literals3() {
    let content = r#"interface Foo {
    x: { a: number };
}
var aaa: Foo;
aaa.x = { /*10*/"#;
    let mut s = Session::new_for_test("completionListInTypedObjectLiterals3", content);
    fourslash::verify_completions_exact_at(&mut s, Some("10"), &["a"]);
    // TODO: }
}
