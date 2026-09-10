use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_same_file() {
    let content = r#"var /*localVariableDefinition*/localVariable;
function /*localFunctionDefinition*/localFunction() { }
class /*localClassDefinition*/localClass { }
interface /*localInterfaceDefinition*/localInterface{ }
module /*localModuleDefinition*/localModule{ export var foo = 1;}


/*localVariableReference*/localVariable = 1;
/*localFunctionReference*/localFunction();
var foo = new /*localClassReference*/localClass();
class fooCls implements /*localInterfaceReference*/localInterface { }
var fooVar = /*localModuleReference*/localModule.foo;"#;
    let mut s = Session::new_for_test("goToDefinitionSameFile", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, false, "localVariableReference", "localFunctionReference", "localC
}
