use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_object_binding_element_property_name01() {
    let content = r#"interface I {
    /*def*/property1: number;
    property2: string;
}

var foo: I;
var { [|/*use*/property1|]: prop1 } = foo;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "use")
}
