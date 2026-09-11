use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_with_optional_properties_generic_partial() {
    let content = r#"// @strict: true
interface Foo {
    a_a: boolean;
    a_b: boolean;
    a_c: boolean;
    b_a: boolean;
}
function partialFoo<T extends Partial<Foo>>(t: T) {return t}
partialFoo({ /*1*/ });"#;
    let mut s = Session::new_for_test("completionsWithOptionalPropertiesGenericPartial", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
