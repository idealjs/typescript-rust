use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_type_only_import() {
    let content = r#"// @Filename: /a.ts
enum /*1*/SyntaxKind { SourceFile }
export type { SyntaxKind }
// @Filename: /b.ts
 export type { SyntaxKind } from './a';
// @Filename: /c.ts
import type { SyntaxKind } from './b';
let kind: [|/*2*/SyntaxKind|];"#;
    let mut s = Session::new_for_test("goToDefinitionTypeOnlyImport", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "2")
}
