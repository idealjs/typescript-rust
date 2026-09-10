use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineInlayHints"]
#[test]
fn inlay_hints_function_parameter_types1() {
    let content = r#"type F1 = (a: string, b: number) => void
const f1: F1 = (a, b) => { }
const f2: F1 = (a, b: number) => { }
function foo1 (cb: (a: string) => void) {}
foo1((a) => { })
function foo2 (cb: (a: Exclude<1 | 2 | 3, 1>) => void) {}
foo2((a) => { })
function foo3 (a: (b: (c: (d: Exclude<1 | 2 | 3, 1>) => void) => void) => void) {}
foo3(a => {
    a(d => {})
})
function foo4<T>(v: T, a: (v: T) => void) {}
foo4(1, a => { })
type F2 = (a: {
    a: number
    b: string
}) => void
const foo5: F2 = (a) => { }"#;
    let mut s = Session::new_for_test("inlayHintsFunctionParameterTypes1", content);
    fourslash::unsupported("VerifyBaselineInlayHints"); // f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
