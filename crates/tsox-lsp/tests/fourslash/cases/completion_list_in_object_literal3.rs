use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_object_literal3() {
    let content = r#"interface IASTNode {
    name: string;
    children: IASTNode[];
}
var ast2: IASTNode = {
    /**/
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
