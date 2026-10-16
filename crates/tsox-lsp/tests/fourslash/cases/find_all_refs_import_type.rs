use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_import_type() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
module.exports = 0;
/*1*/export type /*2*/N = number;
// @Filename: /b.js
type T = import("./a")./*3*/N;"#;
    let _s = Session::new_for_test("findAllRefsImportType", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
