use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_common_js_require2() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
function f() { }
module.exports.f = f
// @Filename: /b.js
const { f } = require('./a')
/**/f"#;
    let mut s = Session::new_for_test("findAllRefsCommonJsRequire2", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
}
