use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_typedef_import_type() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
module.exports = 0;
/** /*1*/@typedef {number} /*2*/Foo */
const dummy = 0;
// @Filename: /b.js
/** @type {import('./a')./*3*/Foo} */
const x = 0;"#;
    let _s = Session::new_for_test("findAllRefsTypedef_importType", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
