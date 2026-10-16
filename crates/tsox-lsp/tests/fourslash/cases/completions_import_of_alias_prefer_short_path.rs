use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_of_alias_prefer_short_path() {
    let content = r#"// @module: commonJs
// @noLib: true
// @Filename: /foo/index.ts
export { foo } from "./lib/foo";
// @Filename: /foo/lib/foo.ts
export const foo = 0;
// @Filename: /user.ts
fo/**/"#;
    let mut s = Session::new_for_test("completionsImport_ofAlias_preferShortPath", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
