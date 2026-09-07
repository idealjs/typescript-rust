use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "d", "c")
}
