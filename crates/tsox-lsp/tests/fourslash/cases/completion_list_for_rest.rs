use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_for_rest() {
    let content = r#"interface Gen {
    x: number;
    parent: Gen;
    millenial: string;
}
let t: Gen;
var { x, ...rest } = t;
rest./*1*/x;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
