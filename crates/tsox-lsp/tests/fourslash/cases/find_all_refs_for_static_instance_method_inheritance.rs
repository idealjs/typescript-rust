use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_for_static_instance_method_inheritance() {
    let content = r#"class X{
	/*0*/foo(): void{}
}

class Y extends X{
	static /*1*/foo(): void{}
}

class Z extends Y{
	static /*2*/foo(): void{}
	/*3*/foo(): void{}
}

const x = new X();
const y = new Y();
const z = new Z();
x.foo();
y.foo();
z.foo();
Y.foo();
Z.foo();"#;
    let mut s = Session::new_for_test("findAllRefsForStaticInstanceMethodInheritance", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "0", "1", "2", "3")
}
