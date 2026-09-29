use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_re_exports2() {
    let content = r#"// @Filename: /a.ts
export function /*1*/foo(): void {}
// @Filename: /b.ts
import { foo as oof } from "./a";"#;
    let mut s = Session::new_for_test("findAllRefsReExports2", content);
    fourslash::verify_no_errors(&mut s, );
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
