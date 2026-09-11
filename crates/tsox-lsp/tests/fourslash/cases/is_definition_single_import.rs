use tsox_lsp::fourslash::{self, Session};


#[test]
fn is_definition_single_import() {
    let content = r#"// @filename: a.ts
export function /*1*/f() {}
// @filename: b.ts
import { /*2*/f } from "./a";"#;
    let mut s = Session::new_for_test("isDefinitionSingleImport", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
}
