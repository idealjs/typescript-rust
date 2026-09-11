use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_import_named_did_not_exist_before() {
    let content = r#"// @noLib: true
// @Filename: /a.ts
export function Test1() {}
export function Test2() {}
// @Filename: /b.ts
import { Test2 } from "./a";
t/**/"#;
    let mut s = Session::new_for_test("completionsImport_named_didNotExistBefore", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
