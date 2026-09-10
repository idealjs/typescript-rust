use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_of_constructor() {
    let content = r#"class A {
    /*aCtr*/constructor(s: string) {}
}
class B extends A { }
class C extends B {
    /*cCtr*/constructor() {
        super("");
    }
}
class D extends B { }
class E implements A { }
const a = new A("a");
const b = new B("b");
const c = new C();
const d = new D("d");
const e = new E();"#;
    let mut s = Session::new_for_test("findAllRefsOfConstructor", content);
    fourslash::verify_no_errors(&mut s, );
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "aCtr", "cCtr")
}
