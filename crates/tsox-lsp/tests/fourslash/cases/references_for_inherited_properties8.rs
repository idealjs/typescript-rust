use tsox_lsp::fourslash::Session;


#[test]
fn references_for_inherited_properties8() {
    let content = r#"interface C extends D {
    /*d*/propD: number;
}
interface D extends C {
    propD: string;
    /*c*/propC: number;
}
var d: D;
d.propD;
d.propC;"#;
    let _s = Session::new_for_test("referencesForInheritedProperties8", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "d", "c")
}
