use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_import_type_js() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: /a.js
/**/module.exports = class C {};
module.exports.D = class D {};
// @Filename: /b.js
/** @type {import("./a")} */
const x = 0;
/** @type {import("./a").D} */
const y = 0;"#;
    let mut s = Session::new_for_test("findAllRefs_importType_js", content);
    fourslash::verify_no_errors(&mut s, );
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "")
}
