use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn references_for_class_members() {
    let content = r#"class Base {
    /*a1*/a: number;
    /*method1*/method(): void { }
}
class MyClass extends Base {
    /*a2*/a;
    /*method2*/method() { }
}

var c: MyClass;
c./*a3*/a;
c./*method3*/method();"#;
    let mut s = Session::new_for_test("referencesForClassMembers", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "a1", "a2", "a3", "method1", "method2", "method3")
}
