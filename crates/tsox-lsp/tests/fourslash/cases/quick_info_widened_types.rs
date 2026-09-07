use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_widened_types() {
    let content = r#"// @strict: false
var /*1*/a = null;                   // var a: any
var /*2*/b = undefined;              // var b: any
var /*3*/c = { x: 0, y: null };	// var c: { x: number, y: any }
var /*4*/d = [null, undefined];      // var d: any[]"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "var a: any", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "var b: any", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "var c: {\n    x: number;\n    y: any;\n}", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "var d: any[]", "")
}
