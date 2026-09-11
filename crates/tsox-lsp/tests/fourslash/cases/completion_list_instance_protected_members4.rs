use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_instance_protected_members4() {
    let content = r#"class Base {
    private privateMethod() { }
    private privateProperty;

    protected protectedMethod() { }
    protected protectedProperty;

    public publicMethod() { }
    public publicProperty;

    protected protectedOverriddenMethod() { }
    protected protectedOverriddenProperty;
}

class C1 extends Base {
    public protectedOverriddenMethod() { }
    public protectedOverriddenProperty;
}

 var c: C1;
 c./*1*/"#;
    let mut s = Session::new_for_test("completionListInstanceProtectedMembers4", content);
    fourslash::verify_completions_exact_at(&mut s, Some("1"), &["protectedOverriddenMethod", "protectedOverriddenProperty", "publicMethod", "publicProperty"]);
}
