use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_type_guard() {
    let content = r#"// @lib: es5
const x = "str";
function assert1(condition: any, msg?: string): /*1*/ ;
function assert2(condition: any, msg?: string): /*2*/ { }
function assert3(condition: any, msg?: string): /*3*/
hi"#;
    let mut s = Session::new_for_test("completionTypeGuard", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
}
