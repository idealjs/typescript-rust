use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_export_crash() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
let a;
module.exports = /**/a;
exports["foo"] = a;"#;
    let mut s = Session::new_for_test("renameExportCrash", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
