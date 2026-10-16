use tsox_lsp::fourslash::{self, Session};


#[test]
fn keyword_shadows_auto_import() {
    let content = r#"
// @Filename: /mod.ts
const value = 1;
export { value as function }

// @Filename: /index.ts
function/**/
"#;
    let mut s = Session::new_for_test("keywordShadowsAutoImport", content);
    // TODO: // The keyword `function` should appear, and the auto-import `function` from ./mod should NOT.
    // TODO: // Includes consumes the keyword match; Excludes then verifies no auto-import `function` remains.
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
