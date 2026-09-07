use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn get_java_script_quick_info4() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
/** @param {[number,string]} [a] */
function /**/f(a) { }"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "", "function f(a?: [number, string]): void", "");
}
