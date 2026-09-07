use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_common_js_require3() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
function f() { }
module.exports = { f }
// @Filename: /b.js
const { f } = require('./a')
/**/f"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "")
}
