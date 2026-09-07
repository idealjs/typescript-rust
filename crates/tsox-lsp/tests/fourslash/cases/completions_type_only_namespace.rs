use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_type_only_namespace() {
    let content = r#"// @Filename: /a.ts
export namespace ns {
  export class Box<T> {}
  export type Type = {};
  export const Value = {};
}
// @Filename: /b.ts
import type { ns } from './a';
let x: ns./**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
