use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn string_completions_vs_escaping() {
    let content = r#"type Value<P extends string> = `var(--\\\\, ${P})`
export const value: Value<'one' | 'two'> = "/*1*/"

export const test: `\ntest\n` = '/*2*/'

export const doubleQuoted1: `"double-quoted"` = '/*3*/'
export const doubleQuoted2: `"double-quoted"` = "/*4*/"

export const singleQuoted2: `'single-quoted'` = "/*5*/"
export const singleQuoted2: `'single-quoted'` = '/*6*/'

export const backtickQuoted1: '`backtick-quoted`' = "/*7*/"
export const backtickQuoted2: '`backtick-quoted`' = `/*8*/`"#;
    let mut s = Session::new_for_test("stringCompletionsVsEscaping", content);
    fourslash::verify_completions_exact_at(&mut s, Some("1"), &["var(--\\\\\\\\\\\\\\\\, one)", "var(--\\\\\\\\\\\\\\\\, two)"]);
    fourslash::verify_completions_exact_at(&mut s, Some("2"), &["\\\\ntest\\\\n"]);
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "4");
    // TODO: f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_exact_at(&mut s, Some("5"), &["'single-quoted'"]);
    fourslash::verify_completions_exact_at(&mut s, Some("6"), &["\\\\'single-quoted\\\\'"]);
    fourslash::verify_completions_exact_at(&mut s, Some("7"), &["`backtick-quoted`"]);
    fourslash::verify_completions_exact_at(&mut s, Some("8"), &["\\\\`backtick-quoted\\\\`"]);
}
