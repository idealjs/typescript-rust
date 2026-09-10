use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_import3() {
    let content = r#"// @Filename: /b.ts
/*2*/export const foo = 1;
// @Filename: /a.ts
import { foo } [|from     /*1*/|] "./b";"#;
    let mut s = Session::new_for_test("goToDefinitionImport3", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1")
}
