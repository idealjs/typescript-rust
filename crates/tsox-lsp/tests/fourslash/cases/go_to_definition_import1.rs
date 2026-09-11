use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_import1() {
    let content = r#"// @Filename: /b.ts
/*2*/export const foo = 1;
// @Filename: /a.ts
import { foo } from      [|"./b/*1*/"|];"#;
    let mut s = Session::new_for_test("goToDefinitionImport1", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
