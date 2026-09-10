use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_list_in_scope() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"namespace TestModule {
    var localVariable = "";
    export var exportedVariable = 0;

    function localFunction() { }
    export function exportedFunction() { }

    class localClass { }
    export class exportedClass { }

    interface localInterface {}
    export interface exportedInterface {}

    namespace localModule {
        export var x = 0;
    }
    export namespace exportedModule {
        export var x = 0;
    }

    var v = /*valueReference*/ 0;
    var t :/*typeReference*/;
}

// Add some new items to the module
namespace TestModule {
    var localVariable2 = "";
    export var exportedVariable2 = 0;

    function localFunction2() { }
    export function exportedFunction2() { }

    class localClass2 { }
    export class exportedClass2 { }

    interface localInterface2 {}
    export interface exportedInterface2 {}

    namespace localModule2 {
        export var x = 0;
    }
    export namespace exportedModule2 {
        export var x = 0;
    }
}
var globalVar: string = "";
function globalFunction() { }

class TestClass {
    property: number;
    method() { }
    staticMethod() { }
    testMethod(param: number) {
        var localVar = 0;
        function localFunction() {};
        /*insideMethod*/
    }
}"#;
    let mut s = Session::new_for_test("completionListInScope", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("valueReference"), &["localVariable", "exportedVariable", "localFunction", "exportedFunction", "localClass", "exportedClass", "localModule", "exportedModule", "exportedVariable2", "exportedFunction2", "exportedClass2", "exportedModule2"], &[]);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "typeReference", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "insideMethod", &fourslash.CompletionsExpectedList{
}
