use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_object_literal7() {
    let content = r#"type Foo = { foo: boolean };
function f<T>(shape: Foo): any;
function f<T>(shape: () => Foo): any;
function f(arg: any) {
  return arg;
}

f({ /*1*/ });
f(() => ({ /*2*/ }));
f(() => (({ /*3*/ })));"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1", "2", "3"}, &fourslash.CompletionsExpectedList{
}
