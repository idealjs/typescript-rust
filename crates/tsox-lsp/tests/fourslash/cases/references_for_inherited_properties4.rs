use tsox_lsp::fourslash::Session;


#[test]
fn references_for_inherited_properties4() {
    let content = r#"class class1 extends class1 {
   /*1*/doStuff() { }
   /*2*/propName: string;
}

var c: class1;
c./*3*/doStuff();
c./*4*/propName;"#;
    let _s = Session::new_for_test("referencesForInheritedProperties4", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
