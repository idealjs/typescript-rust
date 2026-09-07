use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineInlayHints"]
#[test]
fn inlay_hints_function_parameter_types5() {
    let content = r#"declare const STATE_SIGNAL: unique symbol;

declare function test(
  cb: (state: { [STATE_SIGNAL]: unknown }) => void,
): unknown;

test((state) => {});"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineInlayHints"); // f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
