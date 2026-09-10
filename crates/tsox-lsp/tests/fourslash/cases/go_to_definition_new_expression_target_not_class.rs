use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
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
    let mut s = Session::new_for_test("goToDefinitionNewExpressionTargetNotClass", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "invokeExpression1", "invokeExpression2")
}
