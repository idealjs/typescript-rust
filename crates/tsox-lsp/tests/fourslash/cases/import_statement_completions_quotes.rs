use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_statement_completions_quotes() {
    let content = r#"// @Filename: /mod.ts
export const foo = 0;
// @Filename: /single.ts
import * as fs from 'fs';
[|import f/**/|]"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
