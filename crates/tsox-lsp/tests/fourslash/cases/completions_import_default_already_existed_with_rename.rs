use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyApplyCodeActionFromCompletion"]
#[test]
fn completions_import_default_already_existed_with_rename() {
    let content = r#"// @Filename: /a.ts
export default function foo() {}
// @Filename: /b.ts
import f_o_o from "./a";
f/**/;"#;
    let mut s = Session::new_for_test("completionsImport_default_alreadyExistedWithRename", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
