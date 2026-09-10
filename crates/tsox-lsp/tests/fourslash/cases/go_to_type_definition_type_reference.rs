use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToTypeDefinition"]
#[test]
fn go_to_type_definition_type_reference() {
    let content = r#"type User = { name: string };
type Box<T> = { value: T };
declare const boxedUser: Box<User>
/*reference*/boxedUser"#;
    let mut s = Session::new_for_test("goToTypeDefinition_typeReference", content);
    fourslash::unsupported("VerifyBaselineGoToTypeDefinition"); // f.VerifyBaselineGoToTypeDefinition(t, "reference")
}
