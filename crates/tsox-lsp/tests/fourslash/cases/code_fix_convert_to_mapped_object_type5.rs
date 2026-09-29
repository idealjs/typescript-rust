use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_convert_to_mapped_object_type5() {
    let content = r#"type K = "foo" | "bar";
class SomeType {
    [prop: K]: any;
}"#;
    let _s = Session::new_for_test("codeFixConvertToMappedObjectType5", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}
