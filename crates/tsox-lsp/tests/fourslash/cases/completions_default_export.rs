use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_default_export() {
    let content = r#"// @Filename: /a.ts
export default function f() {}
// @Filename: /b.ts
import * as a from "./a";
a./**/;"#;
    let mut s = Session::new_for_test("completionsDefaultExport", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
