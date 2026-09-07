use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_with_optional_properties_generic_partial2() {
    let content = r#"// @strict: true
interface Foo {
    a: boolean;
}
function partialFoo<T extends Partial<Foo>>(x: T, y: T) {return t}
partialFoo({ a: true, b: true }, { /*1*/ });"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
