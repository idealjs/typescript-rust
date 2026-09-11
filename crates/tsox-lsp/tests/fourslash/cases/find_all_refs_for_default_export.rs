use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_for_default_export() {
    let content = r#"// @Filename: a.ts
export default function /*def*/f() {}
// @Filename: b.ts
import /*deg*/g from "./a";
[|/*ref*/g|]();
// @Filename: c.ts
import { f } from "./a";"#;
    let mut s = Session::new_for_test("findAllRefsForDefaultExport", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "def", "deg")
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "ref")
}
