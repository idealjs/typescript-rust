use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyApplyCodeActionFromCompletion"]
#[test]
fn completions_overriding_method8() {
    let content = r#"// @newline: LF
// @Filename: /types1.ts
export interface I { foo: string }
// @Filename: /types2.ts
import { I } from "./types1";
export interface Base { method(p: I): void }
// @Filename: /index.ts
import { Base } from "./types2";
export class C implements Base {
  /**/
}"#;
    let mut s = Session::new_for_test("completionsOverridingMethod8", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
