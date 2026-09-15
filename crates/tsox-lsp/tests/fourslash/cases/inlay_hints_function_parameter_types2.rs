use tsox_lsp::fourslash::Session;


#[test]
fn inlay_hints_function_parameter_types2() {
    let content = r#"class C {}
namespace N { export class Foo {} }
interface Foo {}
function f1(a = 1) {}
function f2(a = "a") {}
function f3(a = true) {}
function f4(a = { } as Foo) {}
function f5(a = <Foo>{}) {}
function f6(a = {} as const) {}
function f7(a = (({} as const))) {}
function f8(a = new C()) {}
function f9(a = new N.C()) {}
function f10(a = ((((new C()))))) {}
function f11(a = { a: 1, b: 1 }) {}
function f12(a = ((({ a: 1, b: 1 })))) {}"#;
    let _s = Session::new_for_test("inlayHintsFunctionParameterTypes2", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
