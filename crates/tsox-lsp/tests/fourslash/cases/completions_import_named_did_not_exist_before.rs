use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_import_named_did_not_exist_before() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noLib: true
// @Filename: /a.ts
export function Test1() {}
export function Test2() {}
// @Filename: /b.ts
import { Test2 } from "./a";
t/**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
