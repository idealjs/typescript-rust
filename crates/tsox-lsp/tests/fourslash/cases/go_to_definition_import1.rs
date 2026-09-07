use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_import1() {
    let content = r#"// @Filename: /b.ts
/*2*/export const foo = 1;
// @Filename: /a.ts
import { foo } from      [|"./b/*1*/"|];"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1")
}
