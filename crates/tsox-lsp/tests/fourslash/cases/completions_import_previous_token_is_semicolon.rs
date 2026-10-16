use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_previous_token_is_semicolon() {
    let content = r#"// @Filename: /a.ts
export function foo() {}
// @Filename: /b.ts
import * as a from 'a';
/**/"#;
    let mut s = Session::new_for_test("completionsImport_previousTokenIsSemicolon", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
