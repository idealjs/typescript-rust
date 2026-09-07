use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"a"}, &fourslash.CompletionsExpectedList{
}
