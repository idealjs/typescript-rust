use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.Insert"]
#[test]
fn format_empty_param_list() {
    let content = r#"function f( f: function){/*1*/"#;
    let mut s = Session::new_for_test("formatEmptyParamList", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("Insert"); // f.Insert(t, "}")
}
