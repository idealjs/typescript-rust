use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.Insert(t, '}')"]
#[test]
fn format_any_type_literal() {
    let content = r#"function foo(x: { } /*objLit*/){
/**/"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.Insert(t, "}")
}
