use tsox_lsp::fourslash::{self, Session};


#[test]
fn references_for_inherited_properties2() {
    let content = r#"interface interface1 {
    /*1*/doStuff(): void;
}

interface interface2 {
    doStuff(): void;
}

interface interface2 extends interface1 {
}

class class1 implements interface2 {
    doStuff() {

    }
}

class class2 extends class1 {

}

var v: class2;
v.doStuff();"#;
    let mut s = Session::new_for_test("referencesForInheritedProperties2", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
