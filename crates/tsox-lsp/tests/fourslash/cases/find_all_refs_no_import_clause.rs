use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_no_import_clause() {
    let content = r#"// @Filename: /a.ts
/*1*/export const /*2*/x = 0;
// @Filename: /b.ts
import "./a";"#;
    let mut s = Session::new_for_test("findAllRefsNoImportClause", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2")
}
