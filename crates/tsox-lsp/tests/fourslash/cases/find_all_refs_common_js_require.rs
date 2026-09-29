use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_common_js_require() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
function f() { }
export { f }
// @Filename: /b.js
const { f } = require('./a')
/**/f"#;
    let _s = Session::new_for_test("findAllRefsCommonJsRequire", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
}
