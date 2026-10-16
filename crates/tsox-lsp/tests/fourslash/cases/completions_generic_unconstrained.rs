use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_generic_unconstrained() {
    let content = r#"// @strict: true
function f<T>(x: T) {
  return x;
}

f({ /**/ });"#;
    let mut s = Session::new_for_test("completionsGenericUnconstrained", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
