use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_instanceof1() {
    let content = r#"// @lib: es5
class /*end*/ C {
}
declare var obj: any;
obj [|/*start*/instanceof|] C;"#;
    let _s = Session::new_for_test("goToDefinitionInstanceof1", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "start")
}
