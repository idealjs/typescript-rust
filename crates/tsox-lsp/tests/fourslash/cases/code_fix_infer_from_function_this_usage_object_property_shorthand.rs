use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_infer_from_function_this_usage_object_property_shorthand() {
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
     returnThisMember,
 };

 container.returnThisMember();"#;
    let mut s = Session::new_for_test("codeFixInferFromFunctionThisUsageObjectPropertyShorthand", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `this: Container`, false, 0, 0)
}
