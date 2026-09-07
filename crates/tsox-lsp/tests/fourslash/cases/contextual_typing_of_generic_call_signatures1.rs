use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn contextual_typing_of_generic_call_signatures1() {
    let content = r#"var f24: {
   <T, U>(x: T): U
};
// x should not be contextually typed 
var f24 = (/**/x) => { return 1 };"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "(parameter) x: any", "")
}
