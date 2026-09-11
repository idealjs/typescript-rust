use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_ambient_class_implement_class_abstract_getters_and_setters() {
    let content = r#"abstract class A {
    abstract get a(): string;
    abstract set a(newName: string);

    abstract get b(): number;

    abstract set c(arg: number | string);
}

declare class C implements A {}"#;
    let mut s = Session::new_for_test("codeFixAmbientClassImplementClassAbstractGettersAndSetters", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
