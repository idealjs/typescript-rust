use tsox_lsp::fourslash::{self, Session};

#[test]
fn rest_params_contextually_typed() {
    let content = r#"var foo: Function = function (/*1*/a, /*2*/b, /*3*/c) { };"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "1", "(parameter) a: any", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(parameter) b: any", "");
    fourslash::verify_quick_info_at(&mut s, "3", "(parameter) c: any", "");
}
