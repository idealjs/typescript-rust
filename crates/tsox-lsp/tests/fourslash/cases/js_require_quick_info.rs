use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn js_require_quick_info() {
    let content = r#"// @allowJs: true
// @Filename: a.js
const /**/x = require("./b");
// @Filename: b.js
exports.x = 0;"#;
    let mut s = Session::new_for_test("jsRequireQuickInfo", content);
    fourslash::verify_quick_info_at(&mut s, "", "import x", "");
}
