use tsox_lsp::fourslash::Session;


#[test]
fn inlay_hints_using() {
    let content = r#"// @target: esnext
using _defer = {
	[Symbol.dispose]() {},
};"#;
    let _s = Session::new_for_test("inlayHintsUsing", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
