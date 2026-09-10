use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_object_binding_pattern08() {
    let content = r#"interface I {
    propertyOfI_1: number;
    propertyOfI_2: string;
}
interface J {
    property1: I;
    property2: string;
}

var foo: J;
var { property1: { propertyOfI_1, /**/ } } = foo;"#;
    let mut s = Session::new_for_test("completionListInObjectBindingPattern08", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["propertyOfI_2"]);
}
