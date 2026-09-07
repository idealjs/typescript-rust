use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_shorthand_property04() {
    let content = r#"interface Foo {
    /*2*/foo(): void
}

let x: Foo = {
    [|f/*1*/oo|]
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1")
}
