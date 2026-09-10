use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn auto_import_provider8() {
    let content = r#"// @Filename: /home/src/workspaces/project/tsconfig.json
{ "compilerOptions": { "lib": ["es5"], "module": "commonjs" } }
// @Filename: /home/src/workspaces/project/package.json
{ "dependencies": { "mylib": "file:packages/mylib" } }
// @Filename: /home/src/workspaces/project/packages/mylib/package.json
{ "name": "mylib", "version": "1.0.0" }
// @Filename: /home/src/workspaces/project/packages/mylib/index.ts
export * from "./mySubDir";
// @Filename: /home/src/workspaces/project/packages/mylib/mySubDir/index.ts
export * from "./myClass";
export * from "./myClass2";
// @Filename: /home/src/workspaces/project/packages/mylib/mySubDir/myClass.ts
export class MyClass {}
// @Filename: /home/src/workspaces/project/packages/mylib/mySubDir/myClass2.ts
export class MyClass2 {}
// @link: /home/src/workspaces/project/packages/mylib -> /home/src/workspaces/project/node_modules/mylib
// @Filename: /home/src/workspaces/project/src/index.ts

const a = new MyClass/*1*/();
const b = new MyClass2/*2*/();"#;
    let mut s = Session::new_for_test("autoImportProvider8", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "1");
    // TODO: opts1158 := f.GetOptions()
    // TODO: opts1158.FormatCodeSettings.NewLineCharacter = "\n"
    fourslash::unsupported("Configure"); // f.Configure(t, opts1158)
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new("1"), &fourslash.ApplyCodeActionFromCompletionOptions{
}
