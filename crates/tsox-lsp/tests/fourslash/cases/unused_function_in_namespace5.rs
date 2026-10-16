use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn unused_function_in_namespace5() {
    let content = r#"// @noUnusedLocals: true
// @noUnusedParameters:true
namespace Validation {
    var function1 = function() {
    }

    export function function2() {

    }

    [| function function3() {
        function1();
    }

    function function4() {

    }

    export let a = function3; |]
}"#;
    let _s = Session::new_for_test("unusedFunctionInNamespace5", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `function function3() {
}
