use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_constructor_of_class_when_class_is_preceded_by_namespace01() {
    let content = r#"namespace Foo {
    export var x;
}

class Foo {
    /*definition*/constructor() {
    }
}

var x = new [|/*usage*/Foo|]();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "usage")
}
