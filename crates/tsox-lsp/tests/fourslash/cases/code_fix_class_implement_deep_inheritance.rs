use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_class_implement_deep_inheritance() {
    let content = r#"// @stableTypeOrdering: true
// @strict: false
// Referenced throughout the inheritance chain.
interface I0 { a: number }

class C1 implements I0 { a: number }
interface I1 { b: number }
interface I2 extends C1, I1 {}

class C2 { c: number }
interface I3 {d: number}
class C3 extends C2 implements I0, I2, I3 {
    a: number;
    b: number;
    d: number;
}

interface I4 { e: number }
interface I5 { f: number }
class C4 extends C3 implements I0, I4, I5 {
    e: number;
    f: number;
}

interface I6 extends C4 {}
class C5 implements I6 {}"#;
    let mut s = Session::new_for_test("codeFixClassImplementDeepInheritance", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
