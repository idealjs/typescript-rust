use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_js_doc_namepath() {
    let content = r#"// @noLib: true
/**
 * @type {module:foo/A} x
 */
var x = 1
var /*0*/A = 0;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, "0")
}
