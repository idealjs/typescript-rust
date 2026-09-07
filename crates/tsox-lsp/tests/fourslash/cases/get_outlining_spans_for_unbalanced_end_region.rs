use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyOutliningSpans"]
#[test]
fn get_outlining_spans_for_unbalanced_end_region() {
    let content = r#"// bottom-heavy region balance
[|// #region matched

// #endregion matched|]

// #endregion unmatched"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyOutliningSpans"); // f.VerifyOutliningSpans(t, lsproto.FoldingRangeKindRegion)
}
