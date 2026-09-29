use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_import3() {
    let content = r#"// @Filename: /b.ts
/*2*/export const foo = 1;
// @Filename: /a.ts
import { foo } [|from     /*1*/|] "./b";"#;
    let _s = Session::new_for_test("goToDefinitionImport3", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
