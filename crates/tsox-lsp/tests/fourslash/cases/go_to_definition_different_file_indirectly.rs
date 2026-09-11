use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_different_file_indirectly() {
    let content = r#"// @Filename: Remote2.ts
var /*remoteVariableDefinition*/rem2Var;
function /*remoteFunctionDefinition*/rem2Fn() { }
class /*remoteClassDefinition*/rem2Cls { }
interface /*remoteInterfaceDefinition*/rem2Int{}
module /*remoteModuleDefinition*/rem2Mod { export var foo; }
// @Filename: Remote1.ts
var remVar;
function remFn() { }
class remCls { }
interface remInt{}
namespace remMod { export var foo; }
// @Filename: Definition.ts
/*remoteVariableReference*/rem2Var = 1;
/*remoteFunctionReference*/rem2Fn();
var rem2foo = new /*remoteClassReference*/rem2Cls();
class rem2fooCls implements /*remoteInterfaceReference*/rem2Int { }
var rem2fooVar = /*remoteModuleReference*/rem2Mod.foo;"#;
    let mut s = Session::new_for_test("goToDefinitionDifferentFileIndirectly", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, false, "remoteVariableReference", "remoteFunctionReference", "remo
}
