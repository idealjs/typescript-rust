use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_mapped_type() {
    let content = r#"/*generic*/type t  < T  > =   {
/*map*/   [   P   in   keyof    T  ]   :   T  [  P  ]
};"#;
    let mut s = Session::new_for_test("formattingMappedType", content);
    // TODO: f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "generic");
    fourslash::verify_current_line_content(&mut s, r#"type t<T> = {"#);
    fourslash::go_to_marker(&mut s, "map");
    fourslash::verify_current_line_content(&mut s, r#"    [P in keyof T]: T[P]"#);
    // TODO: }
}
