use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: {"]
#[test]
fn completions_object_literal_method3() {
    let content = r#"// @newline: LF
// @strictNullChecks: true
// @Filename: a.ts
interface I1 {
    M(x: number): void;
}
interface I2 {
    M(x: number): void;
}
const u: I1 | I2 = {
    /*a*/
}
const i: I1 & I2 = {
    /*b*/
}
interface U1 {
    M(x: number): string;
}
interface U2 {
    M(x: string): number;
}
const o: U1 | U2 = {
    /*c*/
}
interface Op {
    M?(x: number): void;
    N: ((x: string) => void) | null | undefined;
    O?: () => void;
}
const op: Op = {
    /*d*/
}"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
    // TODO: {
    // TODO: {
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "d", &fourslash.CompletionsExpectedList{
}
