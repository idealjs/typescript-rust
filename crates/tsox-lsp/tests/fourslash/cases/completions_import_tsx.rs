use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_tsx() {
    let content = r#"// @noLib: true
// @jsx: preserve
// @Filename: /a.tsx
export type Bar = 0;
export default function Foo() {};
// @Filename: /b.tsx
<Fo/**/ />;"#;
    let mut s = Session::new_for_test("completionsImport_tsx", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
