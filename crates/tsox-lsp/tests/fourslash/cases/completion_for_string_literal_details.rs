use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "path", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "type", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "prop", &fourslash.CompletionsExpectedList{
}
