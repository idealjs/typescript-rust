use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_union_type_property2() {
    let content = r#"interface HasAOrB {
    /*propertyDefinition1*/a: string;
    b: string;
}

interface One {
    common: { /*propertyDefinition2*/a : number; };
}

interface Two {
    common: HasAOrB;
}

var x : One | Two;

x.common.[|/*propertyReference*/a|];"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "propertyReference")
}
