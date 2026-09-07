use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn is_definition_interface_implementation() {
    let content = r#"interface I {
    /*1*/M(): void;
}

class C implements I {
    /*2*/M() { }
}

({} as I).M();
({} as C).M();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2")
}
