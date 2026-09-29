use tsox_lsp::fourslash::Session;


#[test]
fn get_outlining_spans_for_unbalanced_end_region() {
    let content = r#"// bottom-heavy region balance
[|// #region matched

// #endregion matched|]

// #endregion unmatched"#;
    let _s = Session::new_for_test("getOutliningSpansForUnbalancedEndRegion", content);
    // TODO: f.VerifyOutliningSpans(t, lsproto.FoldingRangeKindRegion)
}
