use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_on_jsx_intrinsic_declared_using_catch_call_index_signature() {
    let content = r#"// @jsx: react
// @filename: /a.tsx
declare namespace JSX {
  interface IntrinsicElements { [elemName: string]: any; }
}
</**/div class="democlass" />;"#;
    let mut s = Session::new_for_test("quickInfoOnJsxIntrinsicDeclaredUsingCatchCallIndexSignature", content);
    // TODO: f.VerifyBaselineHover(t)
}
