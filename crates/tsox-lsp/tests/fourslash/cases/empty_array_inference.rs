use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn empty_array_inference() {
    let content = r#"// @strict: false
var x/*1*/x = true ? [1] : [undefined]; 
var y/*2*/y = true ? [1] : [];"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "var xx: number[]", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "var yy: number[]", "")
}
