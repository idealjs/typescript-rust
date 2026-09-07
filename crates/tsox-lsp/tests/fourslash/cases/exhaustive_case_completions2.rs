use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyApplyCodeActionFromCompletion"]
#[test]
fn exhaustive_case_completions2() {
    let content = r#"// @newline: LF
// @Filename: /dep.ts
export enum E {
    A = 0,
    B = "B",
    C = "C",
}
declare const u: E.A | E.B | 1;
export { u };
// @Filename: /main.ts
import { u } from "./dep";
switch (u) {
    case/*1*/
}
// @Filename: /other.ts
import * as d from "./dep";
declare const u: d.E;
switch (u) {
    case/*2*/
}"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new("1"), &fourslash.ApplyCodeActionFromCompletionOptions{
}
