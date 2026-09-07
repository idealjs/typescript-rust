use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn string_completions_vs_escaping() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"type Value<P extends string> = ` + "`" + `var(--\\\\, ${P})` + "`" + `
export const value: Value<'one' | 'two'> = "/*1*/"

export const test: ` + "`" + `\ntest\n` + "`" + ` = '/*2*/'

export const doubleQuoted1: ` + "`" + `"double-quoted"` + "`" + ` = '/*3*/'
export const doubleQuoted2: ` + "`" + `"double-quoted"` + "`" + ` = "/*4*/"

export const singleQuoted2: ` + "`" + `'single-quoted'` + "`" + ` = "/*5*/"
export const singleQuoted2: ` + "`" + `'single-quoted'` + "`" + ` = '/*6*/'

export const backtickQuoted1: '` + "`" + `backtick-quoted` + "`" + `' = "/*7*/"
export const backtickQuoted2: '` + "`" + `backtick-quoted` + "`" + `' = ` + "`" + `/*8*/` + "`" + `"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "6", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "7", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "8", &fourslash.CompletionsExpectedList{
}
