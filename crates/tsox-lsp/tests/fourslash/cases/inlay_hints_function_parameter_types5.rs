use tsox_lsp::fourslash::{self, Session};


#[test]
fn inlay_hints_function_parameter_types5() {
    let content = r#"declare const STATE_SIGNAL: unique symbol;

declare function test(
  cb: (state: { [STATE_SIGNAL]: unknown }) => void,
): unknown;

test((state) => {});"#;
    let mut s = Session::new_for_test("inlayHintsFunctionParameterTypes5", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
