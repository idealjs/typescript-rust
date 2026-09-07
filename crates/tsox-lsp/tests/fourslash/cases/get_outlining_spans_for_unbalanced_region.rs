use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyOutliningSpans"]
#[test]
fn get_outlining_spans_for_unbalanced_region() {
    let content = r#"// top-heavy region balance
// #region unmatched

[|// #region matched

// #endregion matched|]"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyOutliningSpans"); // f.VerifyOutliningSpans(t, lsproto.FoldingRangeKindRegion)
}
