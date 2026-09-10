use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_infer_from_usage_rest_param() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @strict: false
// @noImplicitAny: true
function f(a: number, [|...rest |]){
    a; rest;
}
f(1);
f(2, "s1");
f(3, "s1", "s2");
f(3, "s1", "s2", "s3", "s4");"#;
    let mut s = Session::new_for_test("codeFixInferFromUsageRestParam", content);
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, `...rest: string[]`, false, 0, 0)
}
