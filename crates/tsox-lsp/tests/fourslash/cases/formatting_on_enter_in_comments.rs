use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_on_enter_in_comments() {
    let content = r#"namespace me {
    class A {
        /*
         */*1*/
    /*2*/}
}"#;
    let mut s = Session::new_for_test("formattingOnEnterInComments", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.InsertLine(t, "")
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCurrentLineContent(t, `
}
