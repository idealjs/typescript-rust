use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_inherited_properties2() {
    let content = r#"interface interface1 extends interface1 {
   /*1*/doStuff(): void;   // r0
   /*2*/propName: string;  // r1
}

var v: interface1;
v./*3*/doStuff();  // r2
v./*4*/propName;   // r3"#;
    let mut s = Session::new_for_test("findAllRefsInheritedProperties2", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
