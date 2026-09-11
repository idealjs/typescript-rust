use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_with_optional_properties_generic_partial2() {
    let content = r#"// @strict: true
interface Foo {
    a: boolean;
}
function partialFoo<T extends Partial<Foo>>(x: T, y: T) {return t}
partialFoo({ a: true, b: true }, { /*1*/ });"#;
    let mut s = Session::new_for_test("completionsWithOptionalPropertiesGenericPartial2", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
