use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_rest() {
    let content = r#"interface Gen {
    x: number;
    /*1*/parent: Gen;
    millenial: string;
}
let t: Gen;
var { x, ...rest } = t;
rest.[|/*2*/parent|];"#;
    let _s = Session::new_for_test("goToDefinitionRest", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "2")
}
