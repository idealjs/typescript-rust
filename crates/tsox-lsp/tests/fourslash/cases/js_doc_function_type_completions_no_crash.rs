use tsox_lsp::fourslash::{self, Session};


#[test]
fn js_doc_function_type_completions_no_crash() {
    let content = r#"// @lib: es5
/**
 * @returns {function/**/(): string}
 */
function updateCalendarEvent() {
  return "";
}"#;
    let mut s = Session::new_for_test("jsDocFunctionTypeCompletionsNoCrash", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
