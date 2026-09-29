use tsox_lsp::fourslash::Session;


#[test]
fn get_outlining_spans_for_comments() {
    let content = r#"// @lib: es5
[|/*
    Block comment at the beginning of the file before module:
        line one of the comment
        line two of the comment
        line three
        line four
        line five
*/|]
declare module "m";
[|// Single line comments at the start of the file
// line 2
// line 3
// line 4|]
declare module "n";"#;
    let _s = Session::new_for_test("getOutliningSpansForComments", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyOutliningSpans(t, lsproto.FoldingRangeKindComment)
}
