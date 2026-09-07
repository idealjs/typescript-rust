use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_display_parts_var_with_string_types01() {
    let content = r#"let /*1*/hello: "hello" | 'hello' = "hello";
let /*2*/world: 'world' = "world";
let /*3*/helloOrWorld: "hello" | 'world';"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
