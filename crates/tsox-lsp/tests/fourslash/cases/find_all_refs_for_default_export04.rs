use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_for_default_export04() {
    let content = r#"// @Filename: /a.ts
const /*0*/a = 0;
export /*1*/default /*2*/a;
// @Filename: /b.ts
import /*3*/a from "./a";
/*4*/a;"#;
    let mut s = Session::new_for_test("findAllRefsForDefaultExport04", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "0", "2", "1", "3", "4")
}
