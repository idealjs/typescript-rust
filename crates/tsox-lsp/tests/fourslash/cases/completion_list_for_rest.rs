use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("completionListForRest", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
