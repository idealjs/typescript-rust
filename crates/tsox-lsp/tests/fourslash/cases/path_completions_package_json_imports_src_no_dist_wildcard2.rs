use tsox_lsp::fourslash::{self, Session};


#[test]
fn path_completions_package_json_imports_src_no_dist_wildcard2() {
    let content = r##"// @Filename: /home/src/workspaces/project/tsconfig.json
{
  "compilerOptions": {
    "module": "nodenext",
    "rootDir": "src",
    "outDir": "dist"
  }
}
// @Filename: /home/src/workspaces/project/package.json
{
  "name": "salesforce-pageobjects",
  "version": "1.0.0",
  "imports": {
    "#*": {
      "types": "./dist/*.d.ts",
      "import": "./dist/*.mjs",
      "default": "./dist/*.js"
    }
  }
}
// @Filename: /home/src/workspaces/project/src/action/pageObjects/actionRenderer.ts
export const actionRenderer = 0;
// @Filename: /home/src/workspaces/project/src/index.mts
import { } from "/**/";"##;
    let mut s = Session::new_for_test("pathCompletionsPackageJsonImportsSrcNoDistWildcard2", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "#action/");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "pageObjects/");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
