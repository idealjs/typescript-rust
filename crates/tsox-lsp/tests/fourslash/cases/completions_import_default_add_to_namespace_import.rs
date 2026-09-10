use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyApplyCodeActionFromCompletion"]
#[test]
fn completions_import_default_add_to_namespace_import() {
    let content = r#"// @Filename: /a.ts
export default function foo() {}
// @Filename: /b.ts
import * as a from "./a";
f/**/;"#;
    let mut s = Session::new_for_test("completionsImport_default_addToNamespaceImport", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
