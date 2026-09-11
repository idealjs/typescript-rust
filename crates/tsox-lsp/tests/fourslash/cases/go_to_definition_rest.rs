use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("goToDefinitionRest", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "2")
}
