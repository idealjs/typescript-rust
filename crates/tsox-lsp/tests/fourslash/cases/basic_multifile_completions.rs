use tsox_lsp::fourslash::{self, Session};


#[test]
fn basic_multifile_completions() {
    let content = r#"// @Filename: /a.ts
export const foo = { bar: 'baz' };

// @Filename: /b.ts
import { foo } from './a';
const test = foo./*1*/"#;
    let mut s = Session::new_for_test("basicMultifileCompletions", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
