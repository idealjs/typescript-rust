use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_with_namespace_inside_function() {
    let content = r#"function f() {
    namespace n {
        interface I {
            x: number
        }
        /*1*/
    }
    /*2*/
}
/*3*/
function f2() {
    namespace n2 {
        class I2 {
            x: number
        }
        /*11*/
    }
    /*22*/
}
/*33*/"#;
    let mut s = Session::new_for_test("completionWithNamespaceInsideFunction", content);
    // TODO: f.VerifyCompletions(t, []string{"1", "2", "3"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "11", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "22", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "33", &fourslash.CompletionsExpectedList{
}
