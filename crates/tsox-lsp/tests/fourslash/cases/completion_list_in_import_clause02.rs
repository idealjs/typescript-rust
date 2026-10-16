use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_import_clause02() {
    let content = r#"declare module "M1" {
    export var V;
}

declare module "M2" {
    import { /**/ } from "M1"
}"#;
    let mut s = Session::new_for_test("completionListInImportClause02", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
