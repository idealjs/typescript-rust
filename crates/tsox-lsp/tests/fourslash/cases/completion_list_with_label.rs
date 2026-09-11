use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_with_label() {
    let content = r#" label: while (true) {
    break /*1*/
    continue /*2*/
    testlabel: while (true) {
        break /*3*/
        continue /*4*/
        break tes/*5*/
        continue tes/*6*/
    }
    break /*7*/
    break; /*8*/
}"#;
    let mut s = Session::new_for_test("completionListWithLabel", content);
    // TODO: f.VerifyCompletions(t, []string{"1", "2", "7"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"3", "4", "5", "6"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "8", &fourslash.CompletionsExpectedList{
}
