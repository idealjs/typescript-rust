use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_for_generic_instance1() {
    let content = r#"// @lib: es5
interface Iterator<T, U> {
    (value: T, index: any, list: any): U
}
var i: Iterator<string, number>;
i/**/"#;
    let mut s = Session::new_for_test("completionListForGenericInstance1", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
