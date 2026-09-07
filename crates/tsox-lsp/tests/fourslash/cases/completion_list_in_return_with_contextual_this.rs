use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_return_with_contextual_this() {
    let content = r#"interface Ctx {
    foo(): {
        x: number
    };
}

declare function wrap(cb: (this: Ctx) => any): void;

wrap(function () {
    const xs = this.foo();
    return xs./*inReturn*/
});

wrap(function () {
    const xs = this.foo();
    const y = xs./*involvedInReturn*/
    return y;
});"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "inReturn", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "involvedInReturn", &fourslash.CompletionsExpectedList{
}
