use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_default_import() {
    let content = r#"// @Filename: /a.ts
export default function /*0*/a() {}
// @Filename: /b.ts
import /*1*/a, * as ns from "./a";"#;
    let mut s = Session::new_for_test("findAllRefsDefaultImport", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "0", "1")
}
