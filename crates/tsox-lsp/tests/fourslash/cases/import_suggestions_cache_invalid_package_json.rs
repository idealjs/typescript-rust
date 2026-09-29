use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn import_suggestions_cache_invalid_package_json() {
    let content = r#"// @lib: es5
// @Filename: /home/src/workspaces/project/jsconfig.json
{
  "compilerOptions": {
    "lib": ["es5"],
    "module": "commonjs",
    "types": ["*"]
  },
}
// @Filename: /home/src/workspaces/project/node_modules/@types/node/index.d.ts
declare module 'fs' {
  export function readFile(): void;
}
declare module 'util' {
  export function promisify(): void;
}
// @Filename: /home/src/workspaces/project/package.json
{ "mod" }
// @Filename: /home/src/workspaces/project/a.js

readF/**/"#;
    let mut s = Session::new_for_test("importSuggestionsCache_invalidPackageJson", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
