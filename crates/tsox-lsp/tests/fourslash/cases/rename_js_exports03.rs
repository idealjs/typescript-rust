use tsox_lsp::fourslash::Session;


#[test]
fn rename_js_exports03() {
    let content = r#"// @allowJs: true
// @Filename: a.js
class /*1*/A {
    /*2*/constructor() { }
}
module.exports = A;
// @Filename: b.js
const /*3*/A = require("./a");
new /*4*/A;"#;
    let _s = Session::new_for_test("renameJsExports03", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
