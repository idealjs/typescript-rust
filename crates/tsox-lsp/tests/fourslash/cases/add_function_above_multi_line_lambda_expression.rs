use tsox_lsp::fourslash::{self, Session};

#[test]
fn add_function_above_multi_line_lambda_expression() {
    let content = r#"/**/
() =>
   // do something
0;"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "function Foo() { }");
}
