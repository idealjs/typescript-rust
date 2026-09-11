use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_for_generic_instance1() {
    let content = r#"// @lib: es5
interface Iterator<T, U> {
    (value: T, index: any, list: any): U
}
var i: Iterator<string, number>;
i/**/"#;
    let mut s = Session::new_for_test("completionListForGenericInstance1", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
