use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_shorthand_property05() {
    let content = r#"interface Foo {
    /*3*/foo(): void
}
const /*2*/foo = 1;
let x: Foo = {
    [|f/*1*/oo|]
}"#;
    let mut s = Session::new_for_test("goToDefinitionShorthandProperty05", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1")
}
