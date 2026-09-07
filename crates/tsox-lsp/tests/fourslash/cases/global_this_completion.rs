use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
