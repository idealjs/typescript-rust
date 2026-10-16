use tsox_lsp::fourslash::Session;


#[test]
fn completions_import_from_jsx_tag() {
    let content = r#"// @jsx: react
// @Filename: /types.d.ts
declare namespace JSX {
  interface IntrinsicElements { a }
}
// @Filename: /Box.tsx
export function Box(props: any) { return null; }
// @Filename: /App.tsx
export function App() {
  return (
    <div className="App">
      <Box/**/
    </div>
  )
}"#;
    let _s = Session::new_for_test("completionsImportFromJSXTag", content);
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
