use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_trigger_character() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @jsx: preserve
/** @/*tag*/ */
//</*comment*/
const x: "a" | "b" = "[|/*openQuote*/|]"/*closeQuote*/;
const y: 'a' | 'b' = '[|/*openSingleQuote*/|]'/*closeSingleQuote*/;
const z: 'a' | 'b' = `[|/*openTemplate*/|]`/*closeTemplate*/;
const q: "`a`" | "`b`" = "[|`/*openTemplateInQuote*/a`/*closeTemplateInQuote*/|]";
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
    let mut s = Session::new_for_test("completionsTriggerCharacter", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("tag"), &["param"], &[]);
    fourslash::verify_completions_empty_at(&mut s, Some("comment"));
    // TODO: f.VerifyCompletions(t, "openQuote", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_empty_at(&mut s, Some("closeQuote"));
    // TODO: f.VerifyCompletions(t, "openSingleQuote", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_empty_at(&mut s, Some("closeSingleQuote"));
    // TODO: f.VerifyCompletions(t, "openTemplate", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_empty_at(&mut s, Some("closeTemplate"));
    fourslash::verify_completions_empty_at(&mut s, Some("quoteInComment"));
    fourslash::verify_completions_empty_at(&mut s, Some("lessInComment"));
    fourslash::verify_completions_include_exclude_at(&mut s, Some("openTag"), &["div"], &[]);
    fourslash::verify_completions_empty_at(&mut s, Some("lessThan"));
    fourslash::verify_completions_exact_at(&mut s, Some("closeTag"), &["div>"]);
    fourslash::verify_completions_exact_at(&mut s, Some("path"), &["importMe"]);
    fourslash::verify_completions_empty_at(&mut s, Some("divide"));
}
