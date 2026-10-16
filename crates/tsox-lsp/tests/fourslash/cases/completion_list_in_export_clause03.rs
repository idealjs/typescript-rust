use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_export_clause03() {
    let content = r#"declare module "M1" {
    export var abc: number;
    export var def: string;
}

declare module "M2" {
    export { abc/**/ } from "M1";
}"#;
    let mut s = Session::new_for_test("completionListInExportClause03", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
