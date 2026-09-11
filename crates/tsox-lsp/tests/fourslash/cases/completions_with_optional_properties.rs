use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_with_optional_properties() {
    let content = r#"// @strict: true
interface Options {
    hello?: boolean;
    world?: boolean;
}
declare function foo(options?: Options): void;
foo({
    hello: true,
    /**/
});"#;
    let mut s = Session::new_for_test("completionsWithOptionalProperties", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
