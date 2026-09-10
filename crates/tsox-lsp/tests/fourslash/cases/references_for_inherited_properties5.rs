use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn references_for_inherited_properties5() {
    let content = r#"interface interface1 extends interface1 {
   /*1*/doStuff(): void;
   /*2*/propName: string;
}
interface interface2 extends interface1 {
   doStuff(): void;
   propName: string;
}

var v: interface1;
v.propName;
v.doStuff();"#;
    let mut s = Session::new_for_test("referencesForInheritedProperties5", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2")
}
