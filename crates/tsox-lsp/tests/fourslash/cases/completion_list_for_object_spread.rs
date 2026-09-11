use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_for_object_spread() {
    let content = r#"let o = { a: 1, b: 'no' }
let o2 = { b: 'yes', c: true }
let swap = { a: 'yes', b: -1 };
let addAfter: { a: number, b: string, c: boolean } =
    { ...o, c: false }
let addBefore: { a: number, b: string, c: boolean } =
    { c: false, ...o }
let ignore: { a: number, b: string } =
    { b: 'ignored', ...o }
ignore./*1*/a;
let combinedNestedChangeType: { a: number, b: boolean, c: number } =
    { ...{ a: 1, ...{ b: false, c: 'overriden' } }, c: -1 }
combinedNestedChangeType./*2*/a;
let spreadNull: { a: number } =
    { a: 7, ...null }
let spreadUndefined: { a: number } =
    { a: 7, ...undefined }
spreadNull./*3*/a;
spreadUndefined./*4*/a;"#;
    let mut s = Session::new_for_test("completionListForObjectSpread", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"3", "4"}, &fourslash.CompletionsExpectedList{
}
