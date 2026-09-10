use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_generic_unconstrained() {
    let content = r#"// @strict: true
function f<T>(x: T) {
  return x;
}

f({ /**/ });"#;
    let mut s = Session::new_for_test("completionsGenericUnconstrained", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
