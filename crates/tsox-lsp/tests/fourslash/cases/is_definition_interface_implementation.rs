use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("isDefinitionInterfaceImplementation", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
}
