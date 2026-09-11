use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_infer_from_usage_member2() {
    let content = r#"// @noImplicitAny: true
interface I {
    [|p;|]
}
var i: I;
i.p = 0;"#;
    let mut s = Session::new_for_test("codeFixInferFromUsageMember2", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `p: number;`, false, 0, 0)
}
