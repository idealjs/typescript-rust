use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_infer_from_function_this_usage_object_property_parameter() {
    // TODO: t.Skip("Known failing fourslash test")
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
    let mut s = Session::new_for_test("codeFixInferFromFunctionThisUsageObjectPropertyParameter", content);
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, `this: Container, `, false, 0, 0)
}
