use tsox_lsp::fourslash::Session;


#[test]
fn import_completions_imports_map3() {
    let content = r##"// @Filename: /home/src/workspaces/project/tsconfig.json
{
  "compilerOptions": {
    "module": "nodenext",
    "lib": ["es5"],
    "rootDir": "src",
    "outDir": "dist"
  }
}
// @Filename: /home/src/workspaces/project/package.json
{
  "type": "module",
  "imports": {
    "#internal/": "./dist/internal/"
  }
}
// @Filename: /home/src/workspaces/project/src/internal/foo.ts
export function something(name: string) {}
// @Filename: /home/src/workspaces/project/src/a.ts
import {} from "/*1*/";
import {} from "#internal//*2*/";"##;
    let _s = Session::new_for_test("importCompletions_importsMap3", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"2"}, &fourslash.CompletionsExpectedList{
}
