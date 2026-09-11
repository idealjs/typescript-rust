use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_infer_from_usage_member() {
    let content = r#"// @noImplicitAny: true
class C {
    [|p;|]
    method() {
        this.p.push(10);
    }
}"#;
    let mut s = Session::new_for_test("codeFixInferFromUsageMember", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `p: number[];`, false, 0, 0)
}
