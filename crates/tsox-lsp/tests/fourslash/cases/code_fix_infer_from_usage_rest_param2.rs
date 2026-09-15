use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_infer_from_usage_rest_param2() {
    let content = r#"// @strict: false
// @noImplicitAny: true
function f(a: number, [|...rest |]){
    a; rest;
}
f(1);
f(2, "s1");
f(3, false, "s2");
f(4, "s1", "s2", false, "s4");"#;
    let _s = Session::new_for_test("codeFixInferFromUsageRestParam2", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `...rest: (string | boolean)[]`, false, 0, 0)
}
