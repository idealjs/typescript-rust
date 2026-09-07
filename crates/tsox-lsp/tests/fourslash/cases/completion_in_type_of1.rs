use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_in_type_of1() {
    let content = r#"namespace m1c {
    export interface I { foo(): void; }
}
var x: typeof m1c./*1*/;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", nil)
}
