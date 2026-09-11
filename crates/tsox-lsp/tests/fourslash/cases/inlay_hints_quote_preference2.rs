use tsox_lsp::fourslash::{self, Session};


#[test]
fn inlay_hints_quote_preference2() {
    let content = r#"const a1: "'" = "'";
const b1: "\\" = "\\";
export function fn(a = a1, b = b1) {}"#;
    let mut s = Session::new_for_test("inlayHintsQuotePreference2", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{QuotePreference: lsutil.QuotePre
}
