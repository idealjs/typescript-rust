use tsox_lsp::fourslash::Session;


#[test]
fn completion_inside_function_contains_arguments() {
    let content = r#"function testArguments() {/*1*/}
/*2*/
function testNestedArguments() {
  function nestedfunction(){/*3*/}
}
function f() {
    let g = () => /*4*/
}
let g = () => /*5*/"#;
    let _s = Session::new_for_test("completionInsideFunctionContainsArguments", content);
    // TODO: f.VerifyCompletions(t, []string{"1", "3", "4"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"2", "5"}, &fourslash.CompletionsExpectedList{
}
