use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn rename_js_exports02() {
    let content = r#"// @allowJs: true
// @Filename: a.js
module.exports = class /*1*/A {}
// @Filename: b.js
const /*2*/A = require("./a");"#;
    let mut s = Session::new_for_test("renameJsExports02", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2")
}
