use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_strings_with_trigger_character() {
    let content = r#"type A = "a/b" | "b/a";
const a: A = "[|a/*1*/|]";

type B = "a@b" | "b@a";
const a: B = "[|a@/*2*/|]";

type C = "a.b" | "b.a";
const c: C = "[|a./*3*/|]";

type D = "a'b" | "b'a";
const d: D = "[|a'/*4*/|]";

type E = "a`b" | "b`a";
const e: E = "[|a`/*5*/|]";

type F = 'a"b' | 'b"a';
const f: F = '[|a"/*6*/|]';

type G = "a<b" | "b<a";
const g: G = '[|a</*7*/|]';"#;
    let mut s = Session::new_for_test("completionsStringsWithTriggerCharacter", content);
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
}
