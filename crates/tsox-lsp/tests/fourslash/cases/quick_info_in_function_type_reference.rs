use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_in_function_type_reference() {
    let content = r#"function map(fn: (variab/*1*/le1: string) => void) {
}
var x = <{ (fn: (va/*2*/riable2: string) => void, a: string): void; }> () => { };"#;
    let mut s = Session::new_for_test("quickInfoInFunctionTypeReference", content);
    fourslash::verify_quick_info_at(&mut s, "1", "(parameter) variable1: string", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(parameter) variable2: string", "");
}
