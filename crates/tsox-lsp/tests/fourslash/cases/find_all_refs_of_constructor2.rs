use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_of_constructor2() {
    let content = r#"class A {
    /*a*/constructor(s: string) {}
}
class B extends A {
    /*b*/constructor() { super(""); }
}
class C extends B {
    /*c*/constructor() {
        super();
    }
}
class D extends B { }
const a = new A("a");
const b = new B();
const c = new C();
const d = new D();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "a", "b", "c")
}
