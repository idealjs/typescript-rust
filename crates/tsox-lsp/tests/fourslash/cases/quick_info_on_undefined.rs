use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_on_undefined() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"function foo(a: string) {
}
foo(/*1*/undefined);
var x = {
    undefined: 10
};
x./*2*/undefined = 30;"#;
    let mut s = Session::new_for_test("quickInfoOnUndefined", content);
    fourslash::verify_quick_info_at(&mut s, "1", "var undefined", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(property) undefined: number", "");
}
