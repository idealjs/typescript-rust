use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_display_parts_var_with_string_types01() {
    let content = r#"let /*1*/hello: "hello" | 'hello' = "hello";
let /*2*/world: 'world' = "world";
let /*3*/helloOrWorld: "hello" | 'world';"#;
    let _s = Session::new_for_test("quickInfoDisplayPartsVarWithStringTypes01", content);
    // TODO: f.VerifyBaselineHover(t)
}
