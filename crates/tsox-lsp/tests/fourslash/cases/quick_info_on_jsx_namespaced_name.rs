use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_on_jsx_namespaced_name() {
    let content = r#"// @jsx: react
// @Filename: /types.d.ts
declare namespace JSX {
    interface IntrinsicElements { ['a:b']: { a: string }; }
}
// @filename: /a.tsx
</**/a:b a="accepted" b="rejected" />;"#;
    let mut s = Session::new_for_test("quickInfoOnJsxNamespacedName", content);
    // TODO: f.VerifyBaselineHover(t)
}
