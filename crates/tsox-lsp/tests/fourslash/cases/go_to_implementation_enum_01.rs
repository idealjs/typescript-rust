use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_enum_01() {
    let content = r#"enum [|Foo|] {
    Foo1 = function initializer() { return 5 } (),
    Foo2 = 6
}

Fo/*reference*/o;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "reference")
}
