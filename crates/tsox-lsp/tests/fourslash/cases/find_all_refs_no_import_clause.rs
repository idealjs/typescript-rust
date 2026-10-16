use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_no_import_clause() {
    let content = r#"// @Filename: /a.ts
/*1*/export const /*2*/x = 0;
// @Filename: /b.ts
import "./a";"#;
    let _s = Session::new_for_test("findAllRefsNoImportClause", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
}
