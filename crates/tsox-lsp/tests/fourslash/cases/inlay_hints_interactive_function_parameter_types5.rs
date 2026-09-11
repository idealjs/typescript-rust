use tsox_lsp::fourslash::{self, Session};


#[test]
fn inlay_hints_interactive_function_parameter_types5() {
    let content = r#"const foo: 1n = 1n;
export function fn(b = foo) {}"#;
    let mut s = Session::new_for_test("inlayHintsInteractiveFunctionParameterTypes5", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
