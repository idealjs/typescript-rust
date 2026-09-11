use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_for_object_property() {
    let content = r#"// @Filename: /a.ts
export const foo = { bar: 'baz' };
// @Filename: /b.ts
const test = foo/*1*/
// @Filename: /c.ts
const test2 = {...foo/*2*/}
// @Filename: /d.ts
const test3 = [{...foo/*3*/}]
// @Filename: /e.ts
const test4 = { foo/*4*/ }
// @Filename: /f.ts
const test5 = { foo: /*5*/ }
// @Filename: /g.ts
const test6 = { unrelated: foo/*6*/ }
// @Filename: /i.ts
const test7: { foo/*7*/: "unrelated" }
// @Filename: /h.ts
const test8: { foo: string } = { foo/*8*/ }"#;
    let mut s = Session::new_for_test("completionForObjectProperty", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "4");
    // TODO: f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "5");
    // TODO: f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "6");
    // TODO: f.VerifyCompletions(t, "6", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "7");
    // TODO: f.VerifyCompletions(t, "7", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "8");
    // TODO: f.VerifyCompletions(t, "8", &fourslash.CompletionsExpectedList{
}
