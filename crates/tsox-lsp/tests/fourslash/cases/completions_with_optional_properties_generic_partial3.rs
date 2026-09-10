use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_with_optional_properties_generic_partial3() {
    let content = r#"// @strict: true
interface Foo {
  a: boolean;
}
function partialFoo<T extends Partial<Foo>>(x: T, y: T extends { b?: boolean } ? T & { c: true } : T) {
  return x;
}

partialFoo({ a: true, b: true }, { /*1*/ });"#;
    let mut s = Session::new_for_test("completionsWithOptionalPropertiesGenericPartial3", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
