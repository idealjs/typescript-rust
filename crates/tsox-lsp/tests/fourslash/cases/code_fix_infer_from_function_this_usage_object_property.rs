use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_infer_from_function_this_usage_object_property() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @noImplicitThis: true
function returnThisMember([| |]) {
     return this.member;
 }

 interface Container {
     member: string;
     returnThisMember(): string;
 }

 const container: Container = {
     member: "sample",
     returnThisMember: returnThisMember,
 };

 container.returnThisMember();"#;
    let mut s = Session::new_for_test("codeFixInferFromFunctionThisUsageObjectProperty", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `this: Container`, false, 0, 0)
}
