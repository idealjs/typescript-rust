use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_salsa_methods_on_assigned_function_expressions() {
    let content = r#"// @allowJs: true
// @Filename: something.js
var C = function () { }
/**
 * The prototype method.
 * @param {string} a Parameter definition.
 */
function f(a) {}
C.prototype.m = f;

var x = new C();
x/*1*/.m();"#;
    let _s = Session::new_for_test("quickInfoSalsaMethodsOnAssignedFunctionExpressions", content);
    // TODO: f.VerifyBaselineHover(t)
}
