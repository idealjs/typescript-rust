use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_export_const_equal_to_class() {
    let content = r#"// @Filename: /a.ts
class C {}
export const /*0*/D = C;
// @Filename: /b.ts
import { /*1*/D } from "./a";"#;
    let mut s = Session::new_for_test("findAllRefsExportConstEqualToClass", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "0", "1")
}
