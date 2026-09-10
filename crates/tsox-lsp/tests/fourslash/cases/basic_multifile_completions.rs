use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn basic_multifile_completions() {
    let content = r#"// @Filename: /a.ts
export const foo = { bar: 'baz' };

// @Filename: /b.ts
import { foo } from './a';
const test = foo./*1*/"#;
    let mut s = Session::new_for_test("basicMultifileCompletions", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
