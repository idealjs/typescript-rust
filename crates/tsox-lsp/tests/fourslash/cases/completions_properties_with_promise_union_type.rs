use tsox_lsp::fourslash::Session;


#[test]
fn completions_properties_with_promise_union_type() {
    let content = r#"// @strict: true
type MyType = {
  foo: string;
};
function fakeTest(cb: () => MyType | Promise<MyType>) {}
fakeTest(() => {
  return {
    /*a*/
  };
});"#;
    let _s = Session::new_for_test("completionsPropertiesWithPromiseUnionType", content);
    // TODO: f.VerifyCompletions(t, []string{"a"}, &fourslash.CompletionsExpectedList{
}
