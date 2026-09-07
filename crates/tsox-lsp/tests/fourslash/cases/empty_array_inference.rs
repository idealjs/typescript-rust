use tsox_lsp::fourslash::{self, Session};

#[test]
fn empty_array_inference() {
    let content = r#"// @strict: false
var x/*1*/x = true ? [1] : [undefined]; 
var y/*2*/y = true ? [1] : [];"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "1", "var xx: number[]", "");
    fourslash::verify_quick_info_at(&mut s, "2", "var yy: number[]", "");
}
