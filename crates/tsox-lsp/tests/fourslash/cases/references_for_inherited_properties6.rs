use tsox_lsp::fourslash::Session;


#[test]
fn references_for_inherited_properties6() {
    let content = r#"class class1 extends class1 {
    /*1*/doStuff() { }
}
class class2 extends class1 {
    doStuff() { }
}

var v: class2;
v.doStuff();"#;
    let _s = Session::new_for_test("referencesForInheritedProperties6", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
