use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_global_augmentation2() {
    let content = r#"declare module "A" {
/*1*/                  global                {
    }
}"#;
    let mut s = Session::new_for_test("formattingGlobalAugmentation2", content);
    fourslash::format_document(&mut s, "");
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"    global {"#);
    // TODO: }
}
