use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_infer_from_function_this_usage_object_property_parameter() {
    let content = r#"// @noImplicitThis: true
function returnThisMember([| |]suffix: string) {
     return this.member + suffix;
 }

 interface Container {
     member: string;
     returnThisMember(suffix: string): string;
 }

 const container: Container = {
     member: "sample",
     returnThisMember: returnThisMember,
 };

 container.returnThisMember("");"#;
    let _s = Session::new_for_test("codeFixInferFromFunctionThisUsageObjectPropertyParameter", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `this: Container, `, false, 0, 0)
}
