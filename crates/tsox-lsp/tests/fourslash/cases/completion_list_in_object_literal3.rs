use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_object_literal3() {
    let content = r#"interface IASTNode {
    name: string;
    children: IASTNode[];
}
var ast2: IASTNode = {
    /**/
}"#;
    let mut s = Session::new_for_test("completionListInObjectLiteral3", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["children", "name"]);
}
