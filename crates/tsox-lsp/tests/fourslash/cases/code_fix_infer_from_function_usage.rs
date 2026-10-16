use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_infer_from_function_usage() {
    let content = r#"// @stableTypeOrdering: true
// @noImplicitAny: true
function wrap( [| arr |] ) {
     arr.other(function (a: number, b: number) { return a < b ? -1 : 1 });
 }"#;
    let _s = Session::new_for_test("codeFixInferFromFunctionUsage", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `arr: { other: (arg0: (a: number, b: number) => -1 | 1) => void; }`, fa
}
