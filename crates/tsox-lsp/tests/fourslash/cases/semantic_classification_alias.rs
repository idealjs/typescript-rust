use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySemanticTokens"]
#[test]
fn semantic_classification_alias() {
    let content = r#"// @Filename: /a.ts
export type x = number;
export class y {};
// @Filename: /b.ts
import { /*0*/x, /*1*/y } from "./a";
const v: /*2*/x = /*3*/y;"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/b.ts");
    fourslash::unsupported("VerifySemanticTokens"); // f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
