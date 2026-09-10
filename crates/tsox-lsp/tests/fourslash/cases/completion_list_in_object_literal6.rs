use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_object_literal6() {
    let content = r#"const foo = {
    a: "a",
    b: "b"
};
function fn<T extends { [key: string]: any }>(obj: T, events: { [Key in ` + "`" + `on_${string & keyof T}` + "`" + `]?: Key }) {}

fn(foo, {
    /*1*/
})
fn({ a: "a", b: "b" }, {
    /*2*/
})"#;
    let mut s = Session::new_for_test("completionListInObjectLiteral6", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1", "2"}, &fourslash.CompletionsExpectedList{
}
