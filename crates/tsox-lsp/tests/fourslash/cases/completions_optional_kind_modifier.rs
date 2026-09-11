use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_optional_kind_modifier() {
    let content = r#"interface A { a?: number; method?(): number; };
function f(x: A) {
x./*a*/;
}"#;
    let mut s = Session::new_for_test("completionsOptionalKindModifier", content);
    // TODO: f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
}
