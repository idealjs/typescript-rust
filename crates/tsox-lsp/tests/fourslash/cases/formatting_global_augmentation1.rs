use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_global_augmentation1() {
    let content = r#"/*1*/declare          global                      {
}"#;
    let mut s = Session::new_for_test("formattingGlobalAugmentation1", content);
    // TODO: f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"declare global {"#);
    // TODO: }
}
