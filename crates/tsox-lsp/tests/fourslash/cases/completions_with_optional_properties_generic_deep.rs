use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_with_optional_properties_generic_deep() {
    let content = r#"// @strict: true
interface DeepOptions {
    another?: boolean;
}
interface MyOptions {
    hello?: boolean;
    world?: boolean;
    deep?: DeepOptions
}
declare function bar<T extends MyOptions>(options?: Partial<T>): void;
bar({ deep: {/*1*/} });"#;
    let mut s = Session::new_for_test("completionsWithOptionalPropertiesGenericDeep", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
