use tsox_lsp::fourslash::Session;


#[test]
fn completion_list_in_contextually_typed_argument() {
    let content = r#"interface MyPoint {
    x1: number;
    y1: number;
}

function foo(a: (e: MyPoint) => string) { }
foo((e) => {
    e./*1*/
} );

class test {
    constructor(a: (e: MyPoint) => string) { }
}
var t = new test((e) => {
    e./*2*/
} );"#;
    let _s = Session::new_for_test("completionListInContextuallyTypedArgument", content);
    // TODO: f.VerifyCompletions(t, []string{"1", "2"}, &fourslash.CompletionsExpectedList{
}
