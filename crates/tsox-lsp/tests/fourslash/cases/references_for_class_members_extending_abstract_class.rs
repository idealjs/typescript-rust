use tsox_lsp::fourslash::Session;


#[test]
fn references_for_class_members_extending_abstract_class() {
    let content = r#"abstract class Base {
    abstract /*a1*/a: number;
    abstract /*method1*/method(): void;
}
class MyClass extends Base {
    /*a2*/a;
    /*method2*/method() { }
}

var c: MyClass;
c./*a3*/a;
c./*method3*/method();"#;
    let _s = Session::new_for_test("referencesForClassMembersExtendingAbstractClass", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "a1", "a2", "a3", "method1", "method2", "method3")
}
