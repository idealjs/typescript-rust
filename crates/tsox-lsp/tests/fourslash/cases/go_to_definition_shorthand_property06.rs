use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_shorthand_property06() {
    let content = r#"interface Foo {
    /*2*/foo(): void
}
const foo = 1;
let x: Foo = {
    [|f/*1*/oo|]()
}"#;
    let mut s = Session::new_for_test("goToDefinitionShorthandProperty06", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1")
}
