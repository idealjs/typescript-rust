use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_comments() {
    let content = r#"_.chain()
// wow/*callChain1*/
  .then()
// waa/*callChain2*/
    .then();
wow(
  3,
// uaa/*argument1*/
    4
// wua/*argument2*/
);"#;
    let mut s = Session::new_for_test("formatComments", content);
    // TODO: f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "callChain1");
    fourslash::verify_current_line_content(&mut s, r#"    // wow"#);
    fourslash::go_to_marker(&mut s, "callChain2");
    fourslash::verify_current_line_content(&mut s, r#"    // waa"#);
    fourslash::go_to_marker(&mut s, "argument1");
    fourslash::verify_current_line_content(&mut s, r#"    // uaa"#);
    fourslash::go_to_marker(&mut s, "argument2");
    fourslash::verify_current_line_content(&mut s, r#"    // wua"#);
}
