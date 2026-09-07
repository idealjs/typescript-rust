use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_preferred_suggestions1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"declare let v1: string & {} | "a" | "b" | "c";
v1 = "/*1*/";
declare let v2: number & {} | 0 | 1 | 2;
v2 = /*2*/;
declare let v3: string & Record<never, never> | "a" | "b" | "c";
v3 = "/*3*/";
type LiteralUnion1<T extends U, U> = T | U & {};
type LiteralUnion2<T extends U, U> = T | U & Record<never, never>;
declare let v4: LiteralUnion1<"a" | "b" | "c", string>;
v4 = "/*4*/";
declare let v5: LiteralUnion2<"a" | "b" | "c", string>;
v5 = "/*5*/";"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
}
