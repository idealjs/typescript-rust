use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_infer_from_function_usage() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @stableTypeOrdering: true
// @noImplicitAny: true
function wrap( [| arr |] ) {
     arr.other(function (a: number, b: number) { return a < b ? -1 : 1 });
 }"#;
    let mut s = Session::new_for_test("codeFixInferFromFunctionUsage", content);
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, `arr: { other: (arg0: (a: number, b: number) => -1 | 1) => void; }`, fa
}
