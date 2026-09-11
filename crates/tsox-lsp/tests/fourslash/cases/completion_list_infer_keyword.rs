use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_infer_keyword() {
    let content = r#"type Bar<T> = T extends { a: (x: in/**/) => void }
   ? U
   : never;"#;
    let mut s = Session::new_for_test("completionListInferKeyword", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
