use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("completionListInReturnWithContextualThis", content);
    fourslash::verify_completions_exact_at(&mut s, Some("inReturn"), &["x"]);
    fourslash::verify_completions_exact_at(&mut s, Some("involvedInReturn"), &["x"]);
}
