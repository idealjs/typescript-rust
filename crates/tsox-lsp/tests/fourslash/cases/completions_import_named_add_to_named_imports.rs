use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_named_add_to_named_imports() {
    let content = r#"// @Filename: /a.ts
export function foo() {}
export const x = 0;
// @Filename: /b.ts
import { x } from "./a";
f/**/;"#;
    let mut s = Session::new_for_test("completionsImport_named_addToNamedImports", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
