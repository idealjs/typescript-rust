use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_default_add_to_namespace_import() {
    let content = r#"// @Filename: /a.ts
export default function foo() {}
// @Filename: /b.ts
import * as a from "./a";
f/**/;"#;
    let mut s = Session::new_for_test("completionsImport_default_addToNamespaceImport", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
