use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_different_file() {
    let content = r#"// @Filename: goToDefinitionDifferentFile_Definition.ts
var /*remoteVariableDefinition*/remoteVariable;
function /*remoteFunctionDefinition*/remoteFunction() { }
class /*remoteClassDefinition*/remoteClass { }
interface /*remoteInterfaceDefinition*/remoteInterface{ }
module /*remoteModuleDefinition*/remoteModule{ export var foo = 1;}
// @Filename: goToDefinitionDifferentFile_Consumption.ts
/*remoteVariableReference*/remoteVariable = 1;
/*remoteFunctionReference*/remoteFunction();
var foo = new /*remoteClassReference*/remoteClass();
class fooCls implements /*remoteInterfaceReference*/remoteInterface { }
var fooVar = /*remoteModuleReference*/remoteModule.foo;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, false, "remoteVariableReference", "remoteFunctionReference", "remo
}
