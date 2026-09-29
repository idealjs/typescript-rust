use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_function_overloads() {
    let content = r#"function [|/*functionOverload1*/functionOverload|](value: number);
function /*functionOverload2*/functionOverload(value: string);
function /*functionOverloadDefinition*/functionOverload() {}

[|/*functionOverloadReference1*/functionOverload|](123);
[|/*functionOverloadReference2*/functionOverload|]("123");
[|/*brokenOverload*/functionOverload|]({});"#;
    let _s = Session::new_for_test("goToDefinitionFunctionOverloads", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "functionOverloadReference1", "functionOverloadReference2", 
}
