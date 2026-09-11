use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_js_doc_name_path() {
    let content = r#"// @noLib: true
/**
 * @returns {modu/*1*/le:ControlFlow}
 */
export function cargo() {
}"#;
    let mut s = Session::new_for_test("completionJSDocNamePath", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
