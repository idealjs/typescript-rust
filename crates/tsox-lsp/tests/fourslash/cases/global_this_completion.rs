use tsox_lsp::fourslash::{self, Session};


#[test]
fn global_this_completion() {
    let content = r#"// @allowJs: true
// @target: esnext
// @Filename: test.js
(typeof foo !== "undefined"
  ? foo
  : {}
)./**/;
// @Filename: someLib.d.ts
declare var foo: typeof globalThis;"#;
    let mut s = Session::new_for_test("globalThisCompletion", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
