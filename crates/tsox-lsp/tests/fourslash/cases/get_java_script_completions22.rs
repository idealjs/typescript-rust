use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_java_script_completions22() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: file.js
const abc = {};
({./*1*/});"#;
    let mut s = Session::new_for_test("getJavaScriptCompletions22", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, ".");
    fourslash::verify_completions_empty_at(&mut s, None);
}
