use tsox_lsp::fourslash::{self, Session};

#[test]
fn java_script_modules_error1() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
define('mod1', ['a'], /**/function(a, b) {
	
});"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
}
