use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_object_members_in_type_location_with_typeof() {
    let content = r#"// @strict: true
const languageService = { getCompletions() {} }
type A = Parameters<typeof languageService./*1*/>

declare const obj: { dance: () => {} } | undefined
type B = Parameters<typeof obj./*2*/>"#;
    let mut s = Session::new_for_test("completionListObjectMembersInTypeLocationWithTypeof", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
