use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_for_string_literal_quote_preference7() {
    let content = r#"// @filename: /a.ts
export const a = null;
// @filename: /b.ts
import { a } from './a';

const foo = { '#': null };
foo[|./**/|]"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/b.ts");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
