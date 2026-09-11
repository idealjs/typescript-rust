use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_templates() {
    let content = r#"String.call `${123}`/*1*/
String.call `${123} ${456}`/*2*/"#;
    let mut s = Session::new_for_test("formattingTemplates", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, ";");
    // TODO: f.VerifyCurrentLineContent(t, "String.call`${123}`;")
    fourslash::go_to_marker(&mut s, "2");
    fourslash::insert(&mut s, ";");
    // TODO: f.VerifyCurrentLineContent(t, "String.call`${123} ${456}`;")
}
