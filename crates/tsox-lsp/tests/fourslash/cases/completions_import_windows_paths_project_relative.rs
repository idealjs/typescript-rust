use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_import_windows_paths_project_relative() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: c:/project/tsconfig.json
{
  "compilerOptions": {
    "paths": {
      "~/noIndex/*": ["./src/noIndex/*"],
      "~/withIndex": ["./src/withIndex/index.ts"]
    }
  }
}
// @Filename: c:/project/package.json
{}
// @Filename: c:/project/src/noIndex/a.ts
export const myFunctionA = () => {};
// @Filename: c:/project/src/withIndex/b.ts
export const myFunctionB = () => {};
// @Filename: c:/project/src/withIndex/index.ts
export * from './b';
// @Filename: c:/project/src/reproduction/1.ts
myFunction/**/"#;
    let mut s = Session::new_for_test("completionsImport_windowsPathsProjectRelative", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
