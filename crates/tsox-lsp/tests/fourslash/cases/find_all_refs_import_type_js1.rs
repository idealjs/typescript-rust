use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn find_all_refs_import_type_js1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: /a.js
module.exports = class /**/C {};
module.exports.D = class D {};
// @Filename: /b.js
/** @type {import("./a")} */
const x = 0;
/** @type {import("./a").D} */
const y = 0;"#;
    let mut s = Session::new(content);
    fourslash::verify_no_errors(&mut s);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "")
}
