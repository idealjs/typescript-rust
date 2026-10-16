use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn codefix_infer_from_usage_nullish() {
    let content = r#"// @strict: false
// @noImplicitAny: true
declare const a: string
function wat([|b |]) {
    b(a ?? 1);
}"#;
    let _s = Session::new_for_test("codefixInferFromUsageNullish", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `b: (arg0: string | number) => void`, false, 0, 0)
}
