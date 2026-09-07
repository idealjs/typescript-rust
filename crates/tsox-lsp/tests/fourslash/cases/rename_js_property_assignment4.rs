use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_js_property_assignment4() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
function f() {
   var /*1*/foo = this;
   /*2*/foo.x = 1;
}"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/a.js");
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, "1", "2")
}
