use tsox_lsp::fourslash::{self, Session};


#[test]
fn references_for_inherited_properties9() {
    let content = r#"class D extends C {
    /*1*/prop1: string;
}

class C extends D {
    /*2*/prop1: string;
}

var c: C;
c./*3*/prop1;"#;
    let mut s = Session::new_for_test("referencesForInheritedProperties9", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
