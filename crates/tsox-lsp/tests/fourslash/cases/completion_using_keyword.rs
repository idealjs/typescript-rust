use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    let mut s = Session::new_for_test("completionUsingKeyword", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1", "2", "3", "4"}, &fourslash.CompletionsExpectedList{
}
