use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_sort_case_sensitivity2() {
    let content = r#"// @Filename: /a.ts
export interface HasBar { bar: number }
export function hasBar(x: unknown): x is HasBar { return x && typeof x.bar === "number" }
export function foo() {}
export type __String = string;
// @Filename: /b.ts
import { __String, HasBar, hasBar } from "./a";
f/**/;"#;
    let mut s = Session::new_for_test("autoImportSortCaseSensitivity2", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
