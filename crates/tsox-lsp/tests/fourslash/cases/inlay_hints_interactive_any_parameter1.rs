use tsox_lsp::fourslash::{self, Session};


#[test]
fn inlay_hints_interactive_any_parameter1() {
    let content = r#"function foo (v: any) {}
foo(1);
foo('');
foo(true);
foo(foo);
foo((1));
foo(foo(1));"#;
    let mut s = Session::new_for_test("inlayHintsInteractiveAnyParameter1", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
