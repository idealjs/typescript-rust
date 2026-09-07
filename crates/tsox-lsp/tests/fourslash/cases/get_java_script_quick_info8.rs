use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.Backspace"]
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
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, ".");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("Backspace"); // f.Backspace(t, 1)
    fourslash::go_to_marker(&mut s, "2");
    fourslash::insert(&mut s, ".");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
