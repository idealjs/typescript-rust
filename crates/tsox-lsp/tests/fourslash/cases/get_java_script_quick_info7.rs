use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn get_java_script_quick_info7() {
    // TODO: t.Skip("Known failing fourslash test")
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "function a1(p: any): number", "This is a very cool function that is very
}
