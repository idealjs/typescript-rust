use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn augmented_types_module1() {
    let content = r#"namespace m1c {
    export interface I { foo(): void; }
}
var m1c = 1; // Should be allowed
var x: m1c./*1*/;
var /*2*/r = m1c;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::verify_quick_info_at(&mut s, "2", "var r: number", "");
}
