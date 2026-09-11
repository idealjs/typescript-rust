use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_class_implement_class_member_anonymous_class() {
    let content = r#"// @strict: false
class A {
    foo() {
        return class { x: number; }
    }
    bar() {
        return new class { x: number; }
    }
}
class C implements A {[| |]}"#;
    let mut s = Session::new_for_test("codeFixClassImplementClassMemberAnonymousClass", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}
