use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_convert_to_mapped_object_type5() {
    let content = r#"type K = "foo" | "bar";
class SomeType {
    [prop: K]: any;
}"#;
    let mut s = Session::new_for_test("codeFixConvertToMappedObjectType5", content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
}
