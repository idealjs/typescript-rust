use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_suggestions_cache_export_undefined() {
    let content = r#"// @Filename: /home/src/workspaces/project/tsconfig.json
{ "compilerOptions": { "module": "esnext", "lib": ["es5"] } }
// @Filename: /home/src/workspaces/project/undefined.ts
export = undefined;
// @Filename: /home/src/workspaces/project/undefinedAlias.ts
const x = undefined;
export = x;
// @Filename: /home/src/workspaces/project/index.ts
 /**/"#;
    let mut s = Session::new_for_test("importSuggestionsCache_exportUndefined", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
