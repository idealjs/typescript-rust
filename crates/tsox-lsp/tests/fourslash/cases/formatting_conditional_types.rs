use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_conditional_types() {
    let content = r#"/*L1*/type Diff1<T, U> = T extends U?never:T;
/*L2*/type Diff2<T, U> = T    extends    U  ?    never   :     T;"#;
    let mut s = Session::new_for_test("formattingConditionalTypes", content);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "L1");
    fourslash::verify_current_line_content(&mut s, r#"type Diff1<T, U> = T extends U ? never : T;"#);
    fourslash::go_to_marker(&mut s, "L2");
    fourslash::verify_current_line_content(&mut s, r#"type Diff2<T, U> = T extends U ? never : T;"#);
}
