use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_export_clause02() {
    let content = r#"declare module "M1" {
    export var V;
}
var W;
declare module "M2" {
    export { /**/ } from "M1"
}"#;
    let mut s = Session::new_for_test("completionListInExportClause02", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
