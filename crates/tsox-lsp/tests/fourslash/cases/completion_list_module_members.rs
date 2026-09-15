use tsox_lsp::fourslash::Session;


#[test]
fn completion_list_module_members() {
    let content = r#" namespace Module {
     var innerVariable = 1;
     function innerFunction() { }
     class innerClass { }
     namespace innerModule { }
     interface innerInterface {}
     export var exportedVariable = 1;
     export function exportedFunction() { }
     export class exportedClass { }
     export namespace exportedModule { export var exportedInnerModuleVariable = 1; }
     export interface exportedInterface {}
 }

Module./*ValueReference*/;

var x : Module./*TypeReference*/

class TestClass extends Module./*TypeReferenceInExtendsList*/ { }

interface TestInterface implements Module./*TypeReferenceInImplementsList*/ { }"#;
    let _s = Session::new_for_test("completionListModuleMembers", content);
    // TODO: f.VerifyCompletions(t, []string{"ValueReference", "TypeReferenceInExtendsList"}, &fourslash.Completi
    // TODO: f.VerifyCompletions(t, []string{"TypeReference", "TypeReferenceInImplementsList"}, &fourslash.Comple
}
