use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_js_doc_import_tag5() {
    let content = r#"// @checkJs: true
// @Filename: /a.js
export default function /*0*/a() {}
// @Filename: /b.js
/** @import /*1*/a, * as ns from "./a" */"#;
    let _s = Session::new_for_test("findAllRefsJsDocImportTag5", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "0", "1")
}
