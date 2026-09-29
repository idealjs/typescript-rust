use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_for_string_literal_quote_preference7() {
    let content = r#"// @filename: /a.ts
export const a = null;
// @filename: /b.ts
import { a } from './a';

const foo = { '#': null };
foo[|./**/|]"#;
    let mut s = Session::new_for_test("completionForStringLiteral_quotePreference7", content);
    fourslash::go_to_file(&mut s, "/b.ts");
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
