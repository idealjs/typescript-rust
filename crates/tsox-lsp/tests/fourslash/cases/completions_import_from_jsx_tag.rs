use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyApplyCodeActionFromCompletion"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
