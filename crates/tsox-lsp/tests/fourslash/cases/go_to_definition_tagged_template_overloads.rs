use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_tagged_template_overloads() {
    let content = r#"function /*defFNumber*/f(strs: TemplateStringsArray, x: number): void;
function /*defFBool*/f(strs: TemplateStringsArray, x: boolean): void;
function f(strs: TemplateStringsArray, x: number | boolean) {}

[|/*useFNumber*/f|]`${0}`;
[|/*useFBool*/f|]`${false}`;"#;
    let mut s = Session::new_for_test("goToDefinitionTaggedTemplateOverloads", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "useFNumber", "useFBool")
}
