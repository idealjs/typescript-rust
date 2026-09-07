use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
