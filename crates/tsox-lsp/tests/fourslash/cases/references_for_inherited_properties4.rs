use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn references_for_inherited_properties4() {
    let content = r#"class class1 extends class1 {
   /*1*/doStuff() { }
   /*2*/propName: string;
}

var c: class1;
c./*3*/doStuff();
c./*4*/propName;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
