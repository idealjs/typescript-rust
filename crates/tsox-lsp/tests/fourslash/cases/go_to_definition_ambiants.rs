use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_ambiants() {
    let content = r#"declare var /*ambientVariableDefinition*/ambientVar;
declare function /*ambientFunctionDefinition*/ambientFunction();
declare class ambientClass {
    /*constructorDefinition*/constructor();
    static /*staticMethodDefinition*/method();
    public /*instanceMethodDefinition*/method();
}

/*ambientVariableReference*/ambientVar = 1;
/*ambientFunctionReference*/ambientFunction();
var ambientClassVariable = new /*constructorReference*/ambientClass();
ambientClass./*staticMethodReference*/method();
ambientClassVariable./*instanceMethodReference*/method();"#;
    let mut s = Session::new_for_test("goToDefinitionAmbiants", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, false, "ambientVariableReference", "ambientFunctionReference", "co
}
