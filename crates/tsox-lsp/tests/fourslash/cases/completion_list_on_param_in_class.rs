use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_on_param_in_class() {
    let content = r#"export class encoder {
    static getEncoding(buffer: buffer/**/Pointer
}"#;
    let mut s = Session::new_for_test("completionListOnParamInClass", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
