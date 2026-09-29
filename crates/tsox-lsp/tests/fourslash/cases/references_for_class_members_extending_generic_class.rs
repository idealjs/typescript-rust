use tsox_lsp::fourslash::Session;


#[test]
fn references_for_class_members_extending_generic_class() {
    let content = r#"class Base<T> {
    /*a1*/a: this;
    /*method1*/method<U>(a?:T, b?:U): this { }
}
class MyClass extends Base<number> {
    /*a2*/a;
    /*method2*/method() { }
}

var c: MyClass;
c./*a3*/a;
c./*method3*/method();"#;
    let _s = Session::new_for_test("referencesForClassMembersExtendingGenericClass", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "a1", "a2", "a3", "method1", "method2", "method3")
}
