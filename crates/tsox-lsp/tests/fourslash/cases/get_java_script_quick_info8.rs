use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_java_script_quick_info8() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: file.js
let x = {
	/** @type {number} */
	get m() {
		return undefined;
	}
}
x.m/*1*/;

class Foo {
	/** @type {string} */
	get b() {
		return undefined;
	}
}
var y = new Foo();
y.b/*2*/;"#;
    let mut s = Session::new_for_test("getJavaScriptQuickInfo8", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, ".");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    // TODO: f.Backspace(t, 1)
    fourslash::go_to_marker(&mut s, "2");
    fourslash::insert(&mut s, ".");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
