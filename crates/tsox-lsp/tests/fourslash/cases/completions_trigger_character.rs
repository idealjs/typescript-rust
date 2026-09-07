use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_trigger_character() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @jsx: preserve
/** @/*tag*/ */
//</*comment*/
const x: "a" | "b" = "[|/*openQuote*/|]"/*closeQuote*/;
const y: 'a' | 'b' = '[|/*openSingleQuote*/|]'/*closeSingleQuote*/;
const z: 'a' | 'b' = ` + "`" + `[|/*openTemplate*/|]` + "`" + `/*closeTemplate*/;
const q: "` + "`" + `a` + "`" + `" | "` + "`" + `b` + "`" + `" = "[|` + "`" + `/*openTemplateInQuote*/a` + "`" + `/*closeTemplateInQuote*/|]";
// "/*quoteInComment*/ </*lessInComment*/
// @Filename: /foo/importMe.ts
whatever
// @Filename: /a.tsx
declare global {
    namespace JSX {
        interface Element {}
        interface IntrinsicElements {
            div: {};
        }
    }
}
const ctr = </*openTag*/;
const less = 1 </*lessThan*/;
const closeTag = <div> foo <//*closeTag*/;
import something from "./foo//*path*/";
const divide = 1 //*divide*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "tag", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "comment", nil)
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "openQuote", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "closeQuote", nil)
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "openSingleQuote", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "closeSingleQuote", nil)
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "openTemplate", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "closeTemplate", nil)
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "quoteInComment", nil)
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "lessInComment", nil)
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "openTag", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "lessThan", nil)
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "closeTag", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "path", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "divide", nil)
}
