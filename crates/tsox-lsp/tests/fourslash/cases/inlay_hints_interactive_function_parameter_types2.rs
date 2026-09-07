use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineInlayHints"]
#[test]
fn inlay_hints_interactive_function_parameter_types2() {
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineInlayHints"); // f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
