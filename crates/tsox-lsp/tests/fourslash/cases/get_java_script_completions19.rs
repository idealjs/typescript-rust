use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn get_java_script_completions19() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: file.js
function fn() {
	if (foo) {
		return 0;
	} else {
		return '0';
	}
}
let x = fn();
if(typeof x === 'string') {
	x/*str*/
} else {
	x/*num*/
}"#;
    let mut s = Session::new_for_test("getJavaScriptCompletions19", content);
    fourslash::go_to_marker(&mut s, "str");
    fourslash::insert(&mut s, ".");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "num");
    fourslash::insert(&mut s, ".");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
