use tsox_lsp::fourslash::{self, Session};


#[test]
fn exhaustive_case_completions_untitled_local_enum() {
    let content = r#"// @newline: LF
// @filename: ^/untitled/ts-nul-authority/Untitled-1.ts
enum E {
    A = "A",
    B = "B",
    C = "C",
}
declare const e: E;
switch (e) {
    case/**/
}"#;
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: // Locally defined enum should provide exhaustive case completions in untitled file
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

#[test]
fn exhaustive_case_completions_untitled_global_enum() {
    let content = r#"// @newline: LF
// @filename: /home/src/project/globals.d.ts
declare enum Direction {
	Up = "Up",
	Down = "Down",
	Left = "Left",
	Right = "Right",
}
declare const direction: Direction;

// @filename: ^/untitled/ts-nul-authority/Untitled-1.ts
switch (direction) {
    case/**/
}"#;
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: // Globally declared enum should provide exhaustive case completions in untitled file
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

#[test]
fn exhaustive_case_completions_untitled_string_literals() {
    let content = r#"// @newline: LF
// @filename: ^/untitled/ts-nul-authority/Untitled-1.ts
export {};
declare const status: "pending" | "success" | "error";
switch (status) {
    case/**/
}"#;
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: // String literal unions should provide exhaustive case completions in untitled file
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}

#[test]
fn exhaustive_case_completions_untitled_imported_enum() {
    let content = r#"// @newline: LF
// @filename: /home/src/project/enums.ts
export enum Status {
    Active,
    Inactive,
    Pending,
}

// @filename: ^/untitled/ts-nul-authority/Untitled-1.ts
declare const s: import("/home/src/project/enums").Status;
switch (s) {
    case/**/
}"#;
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
