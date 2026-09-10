use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_get_exports_of_module() {
    let content = r#"declare module "x" {
    declare var x: number;
    export = x;
}

let y: /**/"#;
    let mut s = Session::new_for_test("completionList_getExportsOfModule", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
