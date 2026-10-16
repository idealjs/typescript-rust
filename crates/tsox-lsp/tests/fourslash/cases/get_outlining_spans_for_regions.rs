use tsox_lsp::fourslash::Session;


#[test]
fn get_outlining_spans_for_regions() {
    let content = r#"// @lib: es5
// region without label
[|// #region

// #endregion|]

// region without label with trailing spaces
[|// #region

// #endregion|]

// region with label
[|// #region label1

// #endregion|]

// region with extra whitespace in all valid locations
             [|//              #region          label2    label3

        //        #endregion|]

// No space before directive
[|//#region label4

//#endregion|]

// Nested regions
[|// #region outer

[|// #region inner

// #endregion inner|]

// #endregion outer|]

// region delimiters not valid when there is preceding text on line
 test // #region invalid1

test // #endregion

// region delimiters not valid when in multiline comment
/*
// #region invalid2
*/

/*
// #endregion
*/"#;
    let _s = Session::new_for_test("getOutliningSpansForRegions", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyOutliningSpans(t, lsproto.FoldingRangeKindRegion)
}
