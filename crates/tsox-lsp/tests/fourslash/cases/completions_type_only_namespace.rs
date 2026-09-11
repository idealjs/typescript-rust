use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("completionsTypeOnlyNamespace", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
