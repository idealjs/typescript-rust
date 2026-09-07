use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_with_optional_properties_generic() {
    let content = r#"// @strict: true
interface MyOptions {
    hello?: boolean;
    world?: boolean;
}
declare function bar<T extends MyOptions>(options?: Partial<T>): void;
bar({ hello, /*1*/ });"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
