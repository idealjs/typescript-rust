use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_type_assertion_keywords() {
    let content = r#"// @lib: es5
const a = {
  b: 42 as /*0*/
};

1 as /*1*/

const b = 42 as /*2*/

var c = </*3*/>42"#;
    let mut s = Session::new_for_test("completionsTypeAssertionKeywords", content);
    // TODO: f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
}
