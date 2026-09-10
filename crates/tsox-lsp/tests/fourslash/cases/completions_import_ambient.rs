use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyApplyCodeActionFromCompletion"]
#[test]
fn completions_import_ambient() {
    let content = r#"// @lib: es5
// @module: commonjs
// @Filename: a.d.ts
declare namespace foo { class Bar {} }
declare module 'path1' {
  import Bar = foo.Bar;
  export default Bar;
}
declare module 'path2longer' {
  import Bar = foo.Bar;
  export {Bar};
}

// @Filename: b.ts
Ba/**/"#;
    let mut s = Session::new_for_test("completionsImport_ambient", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
