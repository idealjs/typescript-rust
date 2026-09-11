use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_for_string_literal_quote_preference8() {
    let content = r#"// @filename: /a.ts
export const a = null;
// @filename: /b.ts
import { a } from './a';

const foo = { '"a name\'s all good but it\'s better with more"': null };
foo[|./**/|]"#;
    let mut s = Session::new_for_test("completionForStringLiteral_quotePreference8", content);
    fourslash::go_to_file(&mut s, "/b.ts");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
