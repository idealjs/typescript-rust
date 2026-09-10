use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineInlayHints"]
#[test]
fn inlay_hints_with_closures() {
    let content = r#"function foo1(a: number) {
    return (b: number) => {
        return a + b
    }
}
foo1(1)(2);
function foo2(a: (b: number) => number) {
    return a(1) + 2
}
foo2((c: number) => c + 1);"#;
    let mut s = Session::new_for_test("inlayHintsWithClosures", content);
    fourslash::unsupported("VerifyBaselineInlayHints"); // f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
