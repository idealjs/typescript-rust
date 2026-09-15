use tsox_lsp::fourslash::Session;


#[test]
fn get_outlining_spans_for_regions_no_single_line_folds() {
    let content = r#"// @lib: es5
[|//#region
function foo()[| {

}|]
[|//these
//should|]
//#endregion not you|]
[|// be
// together|]

[|//#region bla bla bla

function bar()[| { }|]

//#endregion|]"#;
    let _s = Session::new_for_test("getOutliningSpansForRegionsNoSingleLineFolds", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyOutliningSpans(t)
}
