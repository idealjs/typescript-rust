use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_interface_heritage_clause_already_have_member() {
    let content = r#"// @strict: false
class Base {
    foo: number;
}

class D extends Base {
    bar: number;
}

interface I {
    foo: number;
    bar: number;
    baz: number;
}

class C extends D implements I { }"#;
    let _s = Session::new_for_test("codeFixClassImplementInterfaceHeritageClauseAlreadyHaveMember", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
