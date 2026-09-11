use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_for_string_literal_details() {
    let content = r#"// @Filename: /other.ts
export const x = 0;
// @Filename: /a.ts
import {} from ".//*path*/";

const x: "a" = "[|/*type*/|]";

interface I {
    /** Prop doc */
    x: number;
    /** Method doc */
    m(): void;
}
declare const o: I;
o["[|/*prop*/|]"];"#;
    let mut s = Session::new_for_test("completionForStringLiteral_details", content);
    // TODO: f.VerifyCompletions(t, "path", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "type", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "prop", &fourslash.CompletionsExpectedList{
}
