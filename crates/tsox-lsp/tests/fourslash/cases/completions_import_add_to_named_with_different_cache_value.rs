use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn completions_import_add_to_named_with_different_cache_value() {
    let content = r#"// @Filename: /home/src/workspaces/project/tsconfig.json
{ "compilerOptions": { "module": "commonjs", "lib": ["es5"] } }
// @Filename: /home/src/workspaces/project/packages/mylib/package.json
{ "name": "mylib", "version": "1.0.0", "main": "index.js" }
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
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "1");
    // TODO: opts1267 := f.GetOptions()
    // TODO: opts1267.FormatCodeSettings.NewLineCharacter = "\n"
    fourslash::unsupported("Configure"); // f.Configure(t, opts1267)
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new("1"), &fourslash.ApplyCodeActionFromCompletionOptions{
    fourslash::unsupported("ReplaceLine"); // f.ReplaceLine(t, 0, "import { MyClass } from \"mylib\";")
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
