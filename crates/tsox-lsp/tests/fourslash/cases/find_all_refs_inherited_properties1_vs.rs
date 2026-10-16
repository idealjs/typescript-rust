use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_inherited_properties1_vs() {
    let content = r#"class class1 extends class1 {
   /*1*/doStuff() { }
   /*2*/propName: string;
}

var v: class1;
v./*3*/doStuff();
v./*4*/propName;"#;
    let _s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyBaselineVSFindAllReferences(t, "1", "2", "3", "4")
}
