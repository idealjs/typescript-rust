use tsox_lsp::fourslash::{self, Session};


#[test]
fn semantic_classification_alias() {
    let content = r#"// @Filename: /a.ts
export type x = number;
export class y {};
// @Filename: /b.ts
import { /*0*/x, /*1*/y } from "./a";
const v: /*2*/x = /*3*/y;"#;
    let mut s = Session::new_for_test("semanticClassificationAlias", content);
    fourslash::go_to_file(&mut s, "/b.ts");
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
