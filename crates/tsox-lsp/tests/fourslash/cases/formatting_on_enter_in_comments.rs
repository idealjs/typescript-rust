use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.VerifyCurrentLineContent(t, `"]
#[test]
fn formatting_on_enter_in_comments() {
    let content = r#"namespace me {
    class A {
        /*
         */*1*/
    /*2*/}
}"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("InsertLine"); // f.InsertLine(t, "")
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCurrentLineContent(t, `
}
