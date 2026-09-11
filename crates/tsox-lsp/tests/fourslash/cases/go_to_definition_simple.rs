use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_simple() {
    let content = r#"// @Filename: Definition.ts
class /*2*/c { }
// @Filename: Consumption.ts
 var n = new [|/*1*/c|]();
 var n = new [|c/*3*/|]();"#;
    let mut s = Session::new_for_test("goToDefinitionSimple", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1", "3")
}
