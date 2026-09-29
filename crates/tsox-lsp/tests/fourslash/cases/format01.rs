use tsox_lsp::fourslash::{self, Session};


#[test]
fn format01() {
    let content = r#"// @lib: es5
/**/namespace Default{var x= ( { } ) ;}"#;
    let mut s = Session::new_for_test("format01", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "");
    fourslash::verify_current_line_content(&mut s, r#"namespace Default { var x = ({}); }"#);
}
