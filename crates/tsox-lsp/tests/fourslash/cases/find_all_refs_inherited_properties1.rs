use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_inherited_properties1() {
    let content = r#"class class1 extends class1 {
   /*1*/doStuff() { }
   /*2*/propName: string;
}

var v: class1;
v./*3*/doStuff();
v./*4*/propName;"#;
    let mut s = Session::new_for_test("findAllRefsInheritedProperties1", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
