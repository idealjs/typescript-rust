use tsox_lsp::fourslash::{self, Session};


#[test]
fn path_completions_package_json_imports_src_no_dist_wildcard3() {
    let content = r##"// @Filename: /home/src/workspaces/project/tsconfig.json
{
  "compilerOptions": {
    "module": "nodenext",
    "rootDir": "src",
    "outDir": "dist",
    "declarationDir": "types"
  }
}
// @Filename: /home/src/workspaces/project/package.json
{
  "types": "index.d.ts",
  "imports": {
    "#component-*": {
      "types@>=4.3.5": "types/components/*.d.ts"
    }
  }
}
// @Filename: /home/src/workspaces/project/nope.ts
export const nope = 0;
// @Filename: /home/src/workspaces/project/src/components/index.ts
export const index = 0;
// @Filename: /home/src/workspaces/project/src/components/blah.ts
export const blah = 0;
// @Filename: /home/src/workspaces/project/src/components/subfolder/one.ts
export const one = 0;
// @Filename: /home/src/workspaces/project/src/a.ts
import { } from "/**/";"##;
    let mut s = Session::new_for_test("pathCompletionsPackageJsonImportsSrcNoDistWildcard3", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "#component-subfolder/");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
