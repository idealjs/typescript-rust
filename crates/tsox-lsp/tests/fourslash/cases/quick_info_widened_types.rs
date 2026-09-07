use tsox_lsp::fourslash::{self, Session};

#[test]
fn quick_info_widened_types() {
    let content = r#"// @strict: false
var /*1*/a = null;                   // var a: any
var /*2*/b = undefined;              // var b: any
var /*3*/c = { x: 0, y: null };	// var c: { x: number, y: any }
var /*4*/d = [null, undefined];      // var d: any[]"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "1", "var a: any", "");
    fourslash::verify_quick_info_at(&mut s, "2", "var b: any", "");
    fourslash::verify_quick_info_at(&mut s, "3", "var c: {\n    x: number;\n    y: any;\n}", "");
    fourslash::verify_quick_info_at(&mut s, "4", "var d: any[]", "");
}
