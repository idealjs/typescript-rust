use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineInlayHints"]
#[test]
fn inlay_hints_interactive_rest_parameters3() {
    let content = r#"function fn(x: number, y: number, a: number, b: number) {
    return x + y + a + b;
}
const foo: [x: number, y: number] = [1, 2];
fn(...foo, 3, 4);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineInlayHints"); // f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
