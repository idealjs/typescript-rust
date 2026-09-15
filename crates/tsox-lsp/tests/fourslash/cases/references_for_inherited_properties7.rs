use tsox_lsp::fourslash::Session;


#[test]
fn references_for_inherited_properties7() {
    let content = r#"class class1 extends class1 {
   /*0*/doStuff() { }
   /*1*/propName: string;
}
interface interface1 extends interface1 {
   /*2*/doStuff(): void;
   /*3*/propName: string;
}
class class2 extends class1 implements interface1 {
   /*4*/doStuff() { }
   /*5*/propName: string;
}

var v: class2;
v.doStuff();
v.propName;"#;
    let _s = Session::new_for_test("referencesForInheritedProperties7", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "0", "1", "2", "3", "4", "5")
}
