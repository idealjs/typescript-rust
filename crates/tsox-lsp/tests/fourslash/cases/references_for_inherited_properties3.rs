use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn references_for_inherited_properties3() {
    let content = r#"interface interface1 extends interface1 {
   /*1*/doStuff(): void;
   /*2*/propName: string;
}

var v: interface1;
v./*3*/propName;
v./*4*/doStuff();"#;
    let mut s = Session::new_for_test("referencesForInheritedProperties3", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
