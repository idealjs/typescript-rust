use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_infer_from_usage_member3() {
    let content = r#"// @noImplicitAny: true
class C {
    constructor([|public p)|] { }
}
new C("string");"#;
    let _s = Session::new_for_test("codeFixInferFromUsageMember3", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `public p: string)`, false, 0, 0)
}
