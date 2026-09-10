use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn references_for_inherited_properties() {
    let content = r#"interface interface1 {
    /*1*/doStuff(): void;
}

interface interface2  extends interface1{
    /*2*/doStuff(): void;
}

class class1 implements interface2 {
    /*3*/doStuff() {

    }
}

class class2 extends class1 {

}

var v: class2;
v./*4*/doStuff();"#;
    let mut s = Session::new_for_test("referencesForInheritedProperties", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
