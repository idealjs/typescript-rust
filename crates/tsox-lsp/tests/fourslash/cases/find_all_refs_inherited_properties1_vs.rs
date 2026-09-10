use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineVSFindAllReferences"]
#[test]
fn find_all_refs_inherited_properties1_vs() {
    let content = r#"class class1 extends class1 {
   /*1*/doStuff() { }
   /*2*/propName: string;
}

var v: class1;
v./*3*/doStuff();
v./*4*/propName;"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyBaselineVSFindAllReferences"); // f.VerifyBaselineVSFindAllReferences(t, "1", "2", "3", "4")
}
