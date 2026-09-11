use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_for_string_literal_in_indexed_access01() {
    let content = r#"interface Foo {
    foo: string;
    bar: string;
}

let x: Foo["[|/*1*/|]"]"#;
    let mut s = Session::new_for_test("completionForStringLiteralInIndexedAccess01", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
