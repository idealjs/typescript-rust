use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineInlayHints"]
#[test]
fn inlay_hints_using() {
    let content = r#"// @target: esnext
using _defer = {
	[Symbol.dispose]() {},
};"#;
    let mut s = Session::new_for_test("inlayHintsUsing", content);
    fourslash::unsupported("VerifyBaselineInlayHints"); // f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
