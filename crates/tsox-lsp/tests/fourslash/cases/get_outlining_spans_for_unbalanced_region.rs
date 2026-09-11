use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_outlining_spans_for_unbalanced_region() {
    let content = r#"// top-heavy region balance
// #region unmatched

[|// #region matched

// #endregion matched|]"#;
    let mut s = Session::new_for_test("getOutliningSpansForUnbalancedRegion", content);
    // TODO: f.VerifyOutliningSpans(t, lsproto.FoldingRangeKindRegion)
}
