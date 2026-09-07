use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_object_binding_element_property_name01() {
    let content = r#"interface I {
    /*1*/property1: number;
    property2: string;
}

var foo: I;
/*2*/var { /*3*/property1: prop1 } = foo;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
