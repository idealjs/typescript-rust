use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn path_completions_package_json_imports_src_no_dist_wildcard4() {
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
  "types": "index.d.ts",
  "imports": {
    "#*": "dist/*",
    "#foo/*": "dist/*",
    "#bar/*": "dist/*",
    "#exact-match": "dist/index.d.ts"
  }
}
// @Filename: /home/src/workspaces/project/nope.ts
export const nope = 0;
// @Filename: /home/src/workspaces/project/src/index.ts
export const index = 0;
// @Filename: /home/src/workspaces/project/src/blah.ts
export const blah = 0;
// @Filename: /home/src/workspaces/project/src/foo/onlyInFooFolder.ts
export const foo = 0;
// @Filename: /home/src/workspaces/project/src/subfolder/one.ts
export const one = 0;
// @Filename: /home/src/workspaces/project/src/a.mts
import { } from "/**/";"##;
    let mut s = Session::new_for_test("pathCompletionsPackageJsonImportsSrcNoDistWildcard4", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "#foo/");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "foo/");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
