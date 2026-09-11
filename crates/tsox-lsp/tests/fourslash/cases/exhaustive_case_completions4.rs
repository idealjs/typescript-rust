use tsox_lsp::fourslash::{self, Session};


#[test]
fn exhaustive_case_completions4() {
    let content = r#"// @lib: es5
// @newline: LF
enum E {
    A = 0,
    B = "B",
    C = "C",
}
// Filtering existing literals
declare const u: E.A | E.B | 1 | 1n | "1";
switch (u) {
    case E.A:
    case 1:
    case 1n:
    case 0x1n:
    case "1":
    case `1`:
    case `1${u}`:
    case/*1*/
}
declare const v: E.A | "1" | "2";
switch (v) {
    case 0:
    case `1`:
    /*2*/
}
// Filtering repeated enum members
enum F {
    A = "A",
    B = "B",
    C = A,
}
declare const x: F;
switch (x) {
    /*3*/
}
// Enum with computed elements
enum G {
    C = 0,
    D = 1 << 1,
    E = 1 << 2,
    OtherD = D,
    DorE = D | E,
}
declare const y: G;
switch (y) {
    /*4*/
}
switch (y) {
    case 0: // same as G.C
    case 1: // same as G.D, but we don't know it
    case 3: // same as G.DorE, but we don't know
    /*5*/
}

// Already exhaustive switch
enum H {
    A = "A",
    B = "B",
    C = "C",
}
declare const z: H;
switch (z) {
    case H.A:
    case H.B:
    case H.C:
    /*6*/
}"#;
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    // TODO: // F.A and F.B (no C because C's value is the same as A's)
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
    // TODO: // No exhaustive case completion offered here because the switch is already exhaustive
    // TODO: f.VerifyCompletions(t, "6", &fourslash.CompletionsExpectedList{
}
