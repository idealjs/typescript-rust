use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_convert_to_mapped_object_type13() {
    let content = r#"let x: {
    [p: ""]: string;
}"#;
    let mut s = Session::new_for_test("codeFixConvertToMappedObjectType13", content);
    // TODO: f.VerifyCodeFixNotAvailable(t, "fixConvertToMappedObjectType")
}
