use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_new_expression_target_not_class() {
    let content = r#"class C2 {
}
let /*I*/I: {
    /*constructSignature*/new(): C2;
};
new [|/*invokeExpression1*/I|]();
let /*symbolDeclaration*/I2: {
};
new [|/*invokeExpression2*/I2|]();"#;
    let _s = Session::new_for_test("goToDefinitionNewExpressionTargetNotClass", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "invokeExpression1", "invokeExpression2")
}
