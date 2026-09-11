use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_with_optional_properties_generic_valid_boolean() {
    let content = r#"// @strict: true
interface MyOptions {
    hello?: boolean;
    world?: boolean;
}
declare function bar<T extends MyOptions>(options?: Partial<T>): void;
bar({ hello: true, /*1*/ });"#;
    let mut s = Session::new_for_test("completionsWithOptionalPropertiesGenericValidBoolean", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
