use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn codefix_infer_from_usage_nullish() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @strict: false
// @noImplicitAny: true
declare const a: string
function wat([|b |]) {
    b(a ?? 1);
}"#;
    let mut s = Session::new_for_test("codefixInferFromUsageNullish", content);
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, `b: (arg0: string | number) => void`, false, 0, 0)
}
