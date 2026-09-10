use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_on_arguments_inside_function() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"function foo(x: string) {
    return /*1*/arguments;
}"#;
    let mut s = Session::new_for_test("quickInfoOnArgumentsInsideFunction", content);
    fourslash::verify_quick_info_at(&mut s, "1", "(local var) arguments: IArguments", "");
}
