use tsox_lsp::fourslash::Session;


#[test]
fn completion_using_keyword() {
    let content = r#"function foo() {
    usin/*1*/
}
async function bar() {
    await usin/*2*/
}

class C {
    foo() {
        usin/*3*/
    }

    async bar() {
        await usin/*4*/
    }
}"#;
    let _s = Session::new_for_test("completionUsingKeyword", content);
    // TODO: f.VerifyCompletions(t, []string{"1", "2", "3", "4"}, &fourslash.CompletionsExpectedList{
}
