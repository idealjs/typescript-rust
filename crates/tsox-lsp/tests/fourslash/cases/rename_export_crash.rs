use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_export_crash() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
let a;
module.exports = /**/a;
exports["foo"] = a;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
