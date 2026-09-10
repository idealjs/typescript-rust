use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_instanceof1() {
    let content = r#"// @lib: es5
class /*end*/ C {
}
declare var obj: any;
obj [|/*start*/instanceof|] C;"#;
    let mut s = Session::new_for_test("goToDefinitionInstanceof1", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "start")
}
