use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_default_type_argument_position_type_only() {
    let content = r#"// @lib: es5
const foo = "foo";
function test1<T = /*1*/>() {}"#;
    let mut s = Session::new_for_test("completionListDefaultTypeArgumentPositionTypeOnly", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
