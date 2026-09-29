use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn get_java_script_quick_info7() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: file.js
/**
 * This is a very cool function that is very nice.
 * @returns something
 * @param p anotherthing
 */
function a1(p) {
	try {
		throw new Error('x');
	} catch (x) { x--; }
	return 23;
}

x - /**/a1()"#;
    let mut s = Session::new_for_test("getJavaScriptQuickInfo7", content);
    fourslash::verify_quick_info_at(&mut s, "", "function a1(p: any): number", "This is a very cool function that is very nice.");
}
