use tsox_lsp::fourslash::{self, Session};

#[test]
fn contextual_typing_of_generic_call_signatures1() {
    let content = r#"var f24: {
   <T, U>(x: T): U
};
// x should not be contextually typed 
var f24 = (/**/x) => { return 1 };"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "", "(parameter) x: any", "");
}
