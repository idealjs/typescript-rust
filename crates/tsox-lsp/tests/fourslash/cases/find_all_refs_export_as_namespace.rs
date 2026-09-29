use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_export_as_namespace() {
    let content = r#"// @Filename: /node_modules/a/index.d.ts
export function /*0*/f(): void;
export as namespace A;
// @Filename: /b.ts
import { /*1*/f } from "a";
// @Filename: /c.ts
A./*2*/f();"#;
    let mut s = Session::new_for_test("findAllRefsExportAsNamespace", content);
    fourslash::verify_no_errors(&mut s, );
    // TODO: f.VerifyBaselineFindAllReferences(t, "0", "1", "2")
}
