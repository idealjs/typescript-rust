use tsox_lsp::fourslash::{self, Session};


#[test]
fn inlay_hints_this_parameter() {
    let content = r#"interface I {
    a: number;
}

declare function fn(
    callback: (a: number, b: string) => void
): void;


fn(function (this, a, b) { });
fn(function (this: I, a, b) { });"#;
    let mut s = Session::new_for_test("inlayHintsThisParameter", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
