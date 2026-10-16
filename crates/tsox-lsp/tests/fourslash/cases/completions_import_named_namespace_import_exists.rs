use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_named_namespace_import_exists() {
    let content = r#"// @Filename: /a.ts
export function foo() {}
// @Filename: /b.ts
import * as a from "./a";
f/**/;"#;
    let mut s = Session::new_for_test("completionsImport_named_namespaceImportExists", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
