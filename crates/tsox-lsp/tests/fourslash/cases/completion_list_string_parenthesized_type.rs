use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_string_parenthesized_type() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"type T1 = "a" | "b" | "c";
type T2<T extends T1> = {};

type T3 = T2<"[|/*1*/|]">;
type T4 = T2<("[|/*2*/|]")>;
type T5 = T2<(("[|/*3*/|]"))>;
type T6 = T2<((("[|/*4*/|]")))>;

type T7<P extends T1, K extends T1> = {};
type T8 = T7<"a", ((("[|/*5*/|]")))>;

interface Foo {
    a: number;
    b: number;
}
const a: Foo["[|/*6*/|]"];
const b: Foo[("[|/*7*/|]")];
const b: Foo[(("[|/*8*/|]"))];"#;
    let mut s = Session::new_for_test("completionListStringParenthesizedType", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "6", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "7", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "8", &fourslash.CompletionsExpectedList{
}
