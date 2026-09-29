use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_jsx_opening_tag_import_default() {
    let content = r#"// @module: commonjs
// @jsx: react
// @Filename: /component.tsx
export default function (props: any) {}
// @Filename: /index.tsx
export function Index() {
    return <Component/**/
}"#;
    let mut s = Session::new_for_test("completionsImport_jsxOpeningTagImportDefault", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
