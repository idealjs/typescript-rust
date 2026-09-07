use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_satisfies_expression1() {
    let content = r#"const STRINGS = {
    [|/*definition*/title|]: 'A Title',
} satisfies Record<string,string>;

//somewhere in app
STRINGS.[|/*usage*/title|]"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "definition", "usage")
}
