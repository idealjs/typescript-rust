use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_re_export_star() {
    let content = r#"// @Filename: /a.ts
export function /*0*/foo(): void {}
// @Filename: /b.ts
export * from "./a";
// @Filename: /c.ts
import { /*1*/foo } from "./b";"#;
    let mut s = Session::new_for_test("findAllRefsReExportStar", content);
    fourslash::verify_no_errors(&mut s, );
    // TODO: f.VerifyBaselineFindAllReferences(t, "0", "1")
}
