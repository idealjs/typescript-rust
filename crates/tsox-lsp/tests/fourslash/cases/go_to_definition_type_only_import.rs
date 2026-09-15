use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("goToDefinitionTypeOnlyImport", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "2")
}
